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
use nbt::codec::NbtCodec;
use packets::*;
use std::net::TcpStream;
use uuid::Uuid;

pub struct MinebotClient {
    sock: TcpStream,
    codec: NbtCodec,
    gamestate: GameState
}

impl MinebotClient {
    pub fn connect(host: String, port: u16, username: String) -> Result<Self> {
        info!("Connecting to {}:{}...", host, port);
        let mut sock = TcpStream::connect((&host as &str, port))?;
        let mut codec = NbtCodec::new();

        let packet = HandshakePacket::HandshakePacket {
            version: 340,
            host: host,
            port: port,
            next: 2
        };
        trace!("Sending: {:?}", packet);
        codec.send(&mut sock, packet)?;

        let packet = ClientLoginPacket::LoginStart {
            name: username.clone()
        };
        trace!("Sending: {:?}", packet);
        codec.send(&mut sock, packet)?;

        let packet = codec.receive(&mut sock)?;
        trace!("Received: {:?}", packet);
        let uuid = match packet {
            ServerLoginPacket::LoginSuccess { uuid, .. } => {
                uuid
            }
        };
        info!("Successfully connected, player id is {}", uuid);

        let gamestate = GameState::new(Uuid::parse_str(uuid.as_ref()).unwrap(), username);
        let mut res = MinebotClient {
            sock,
            codec: NbtCodec::new(),
            gamestate
        };

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

    fn send(&mut self, packet: ClientPacket) -> Result<()> {
        trace!("Sending: {:?}", packet);
        self.codec.send(&mut self.sock, &packet)?;
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
        self.gamestate.handle_packet(packet);
        match *packet {
            ServerPacket::KeepAlive { id } => {
                self.send(ClientPacket::KeepAlive {
                    id: id
                })?;
            }
            ServerPacket::PlayerPositionAndLook { teleport_id, .. } => {
                if teleport_id != 0 {
                    self.send(ClientPacket::TeleportConfirm {
                        teleport_id: teleport_id
                    })?;
                }
                self.send_position()?;
            }
            ServerPacket::UpdateHealth { .. } => {
                if self.gamestate.health() == 0.0 {
                    self.send(ClientPacket::ClientStatus {
                        action_id: 0
                    })?;
                }
            }
            _ => {}
        };
        Ok(())
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

    pub fn health(&self) -> f32 {
        self.gamestate.health()
    }

    pub fn food(&self) -> f32 {
        self.gamestate.food()
    }

    pub fn my_position(&self) -> &Position {
        self.gamestate.my_orientation().position()
    }

    pub fn say<M: Into<String>>(&mut self, msg: M) -> Result<()> {
        self.send(ClientPacket::ChatMessage { message: msg.into() })
    }

    fn send_position(&mut self) -> Result<()> {
        let orientation = self.gamestate.my_orientation();
        self.send(ClientPacket::PlayerPositionAndLook {
            x: orientation.x(),
            y: orientation.y(),
            z: orientation.z(),
            yaw: orientation.yaw(),
            pitch: orientation.pitch(),
            on_ground: true
        })
    }

    pub fn block_state_at(&self, position: &BlockPosition) -> Option<BlockState> {
        self.gamestate.block_state_at(position)
    }

    pub fn find_block_ids_within(&self, block_id: u16, position: &BlockPosition, distance: i32) -> Vec<BlockPosition> {
        self.gamestate.find_block_ids_within(block_id, position, distance)
    }

    pub fn find_path_to(&self, start: BlockPosition, dest: BlockPosition) -> Option<Vec<BlockPosition>> {
        self.gamestate.find_path_to(start, dest)
    }

    pub fn player_names(&self) -> Vec<&str> {
        self.gamestate.player_names()
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