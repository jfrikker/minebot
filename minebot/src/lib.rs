#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;

pub mod blocks;
pub mod events;
mod gamestate;
pub mod geom;

use blocks::BlockState;
use events::{Event, EventMatchers};
use gamestate::GameState;
use geom::{BlockPosition, Position};
use nbt::{NbtDecode, NbtEncode};
use nbt::codec::NbtCodec;
use packets::*;
use std::fmt::Debug;
use std::net::TcpStream;

pub struct MinebotClient {
    sock: TcpStream,
    codec: NbtCodec,
    gamestate: GameState
}

impl MinebotClient {
    pub fn connect(host: String, port: u16, username: String) -> Result<Self> {
        info!("Connecting to {}:{}...", host, port);
        let sock = TcpStream::connect((&host as &str, port))?;
        let mut gamestate = GameState::default();
        gamestate.username = username.clone();
        gamestate.health = 10.0;
        gamestate.food = 10.0;

        let mut res = MinebotClient {
            sock,
            codec: NbtCodec::new(),
            gamestate
        };

        res.send(
            HandshakePacket::HandshakePacket {
                version: 340,
                host: host,
                port: port,
                next: 2
            }
        )?;

        res.send(
            ClientLoginPacket::LoginStart {
                name: username
            }
        )?;

        let response: ServerLoginPacket = res.receive_any()?;

        match response {
            ServerLoginPacket::LoginSuccess {username: _, uuid} => info!("Successfully connected, player id is {}", uuid)
        }

        res.poll_until(|packet| 
            match packet {
                ServerPacket::PlayerAbilities{ .. } => true,
                _ => false
            }
        )?;

        res.send(ClientPacket::ClientSettings {
            locale: "en-US".into(),
            view_distance: 4,
            chat_mode: 0,
            chat_colors: false,
            displayed_skin: 0xFF,
            main_hand: 0
        })?;

        res.poll_until(|packet| 
            match packet {
                ServerPacket::KeepAlive{ .. } => true,
                _ => false
            }
        )?;

        Ok(res)
    }

    fn send<P: NbtEncode + Debug>(&mut self, packet: P) -> Result<()> {
        trace!("Sending: {:?}", packet);
        self.codec.send(&mut self.sock, packet)?;
        Ok(())
    }

    pub fn poll(&mut self) -> Result<ServerPacket> {
        let packet: ServerPacket = self.receive()?;
        self.handle(&packet)?;
        Ok(packet)
    }

    pub fn poll_until<F>(&mut self, pred: F) -> Result<ServerPacket>
        where F: Fn(&ServerPacket) -> bool {
        let mut packet = self.poll()?;
        while !pred(&packet) {
            packet = self.poll()?;
        }
        Ok(packet)
    }

    pub fn poll_until_event(&mut self, matchers: &EventMatchers) -> Result<Event> {
        let mut packet: ServerPacket = self.receive()?;
        let mut event = matchers.match_packet(&packet, &self.gamestate);
        self.handle(&packet)?;
        while event.is_none() {
            packet = self.receive()?;
            event = matchers.match_packet(&packet, &self.gamestate);
            self.handle(&packet)?;
        }
        Ok(event.unwrap())
    }

    fn handle(&mut self, packet: &ServerPacket) -> Result<()> {
        match *packet {
            ServerPacket::BlockChange { position, block_state } => {
                let pos = BlockPosition::new((position >> 38) as i32, (position >> 26 & 0xFFF) as i32, (position & 0x3FFFFFF) as i32);
                let bs = BlockState(block_state);
                self.gamestate.set_block_state(&pos, bs);
            }
            ServerPacket::ChunkData { chunk_x, chunk_z, full_chunk, primary_bitmask, ref data } => {
                if full_chunk {
                    self.gamestate.load_chunk_data(chunk_x, chunk_z, primary_bitmask as u8, data)
                }
            }
            ServerPacket::JoinGame { entity_id, .. } => {
                self.gamestate.my_entity_id = entity_id;
            }
            ServerPacket::KeepAlive { id } => {
                self.send(ClientPacket::KeepAlive {
                    id: id
                })?;
            }
            ServerPacket::PlayerPositionAndLook {x, y, z, yaw, pitch, flags, teleport_id, .. } => {
                if teleport_id != 0 {
                    self.send(ClientPacket::TeleportConfirm {
                        teleport_id: teleport_id
                    })?;
                }
                if flags & 0x01 != 0 {
                    self.gamestate.my_orientation.add_x(x);
                } else {
                    self.gamestate.my_orientation.set_x(x);
                }
                if flags & 0x02 != 0 {
                    self.gamestate.my_orientation.add_y(y);
                } else {
                    self.gamestate.my_orientation.set_y(y);
                }
                if flags & 0x04 != 0 {
                    self.gamestate.my_orientation.add_z(z);
                } else {
                    self.gamestate.my_orientation.set_z(z);
                }
                if flags & 0x08 != 0 {
                    self.gamestate.my_orientation.add_yaw(yaw);
                } else {
                    self.gamestate.my_orientation.set_yaw(yaw);
                }
                if flags & 0x10 != 0 {
                    self.gamestate.my_orientation.add_pitch(pitch);
                } else {
                    self.gamestate.my_orientation.set_pitch(pitch);
                }
                self.send_position()?;
            }
            ServerPacket::UnloadChunk { chunk_x, chunk_z } => {
                self.gamestate.unload_chunk(chunk_x, chunk_z);
            }
            ServerPacket::UpdateHealth { health, food, .. } => {
                self.gamestate.health = health / 2.0;
                self.gamestate.food = (food as f32) / 2.0;

                if health == 0.0 {
                    self.send(ClientPacket::ClientStatus {
                        action_id: 0
                    })?;
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn receive_any<P: NbtDecode + Debug>(&mut self) -> Result<P> {
        let packet = self.codec.receive(&mut self.sock)?;
        trace!("Received: {:?}", packet);
        Ok(packet)
    }

    fn receive(&mut self) -> Result<ServerPacket> {
        let packet = self.codec.receive(&mut self.sock)?;
        match &packet {
            ServerPacket::ChunkData { chunk_x, chunk_z, .. } => {
                trace!("Received: ChunkData {{ chunk_x: {}, chunk_z: {}, ... }}", chunk_x, chunk_z);
            }
            p => trace!("Received: {:?}", p)
        }
        Ok(packet)
    }

    pub fn get_health(&self) -> f32 {
        self.gamestate.health
    }

    pub fn get_food(&self) -> f32 {
        self.gamestate.food
    }

    pub fn get_my_position(&self) -> &Position {
        &self.gamestate.my_orientation.position()
    }

    pub fn say<M: Into<String>>(&mut self, msg: M) -> Result<()> {
        self.send(ClientPacket::ChatMessage { message: msg.into() })
    }

    fn send_position(&mut self) -> Result<()> {
        self.send(ClientPacket::PlayerPositionAndLook {
            x: self.gamestate.my_orientation.x(),
            y: self.gamestate.my_orientation.y(),
            z: self.gamestate.my_orientation.z(),
            yaw: self.gamestate.my_orientation.yaw(),
            pitch: self.gamestate.my_orientation.pitch(),
            on_ground: true
        })
    }

    pub fn get_block_state_at(&self, position: &BlockPosition) -> Option<BlockState> {
        self.gamestate.get_block_state_at(position)
    }

    pub fn find_block_ids_within(&self, block_id: u16, position: &BlockPosition, distance: i32) -> Vec<BlockPosition> {
        self.gamestate.find_block_ids_within(block_id, position, distance)
    }

    pub fn find_path_to(&self, start: BlockPosition, dest: BlockPosition) -> Option<Vec<BlockPosition>> {
        self.gamestate.find_path_to(start, dest)
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: std::io::Error) {
            description(err.description())
            cause(err)
            from()
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;