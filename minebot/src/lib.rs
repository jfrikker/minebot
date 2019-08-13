#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;

pub mod blocks;
mod clock;
pub mod events;
mod gamestate;
pub mod geom;

use blocks::BlockState;
use clock::Clock;
use events::EventMatcher;
use gamestate::GameState;
use geom::{BlockPosition, Position};
use nbt::codec::NbtCodec;
use packets::*;
use std::net::TcpStream;
use uuid::Uuid;

pub struct MinebotClient {
    sock: TcpStream,
    codec: NbtCodec,
    gamestate: GameState,
    clock: Clock,
    initialized: bool
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
            gamestate,
            clock: Clock::default(),
            initialized: false
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

        res.initialized = true;

        Ok(res)
    }

    fn send(&mut self, packet: ClientPacket) -> Result<()> {
        trace!("Sending: {:?}", packet);
        self.codec.send(&mut self.sock, &packet)?;
        Ok(())
    }

    pub fn poll(&mut self) -> Result<Option<ServerPacket>> {
        let packet = self.receive()?;
        match packet.as_ref() {
            Some(got_packet) => {
                self.handle(got_packet)?;
            }
            None => {
                if self.initialized && self.gamestate.handle_tick() {
                    self.send_position()?;
                }
            }
        }
        Ok(packet)
    }

    pub fn poll_until<F>(&mut self, pred: F) -> Result<ServerPacket>
        where F: Fn(&ServerPacket) -> bool {
        let mut packet = self.poll()?;
        loop {
            if let Some(got_packet) = packet {
                if pred(&got_packet) {
                    return Ok(got_packet);
                }
            }
            packet = self.poll()?;
        }
    }

    pub fn poll_until_event<M: EventMatcher>(&mut self, matchers: &M) -> Result<M::Event> {
        let mut packet = self.receive()?;
        loop {
            match packet {
                Some(got_packet) => {
                    let event = matchers.match_packet(&got_packet, &self.gamestate);
                    self.handle(&got_packet)?;
                    if let Some(evt) = event { 
                        return Ok(evt);
                    }
                },
                None => {
                    if self.initialized && self.gamestate.handle_tick() {
                        self.send_position()?;
                    }
                    if let Some(evt) = matchers.match_tick(self.clock.current_tick()) {
                        return Ok(evt);
                    }
                }
            }
            packet = self.receive()?;
        }
    }

    fn handle(&mut self, packet: &ServerPacket) -> Result<()> {
        self.gamestate.handle_packet(packet);
        self.clock.handle_packet(packet);
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

    fn receive(&mut self) -> Result<Option<ServerPacket>> {
        if self.clock.advance() {
            return Ok(None);
        }
        
        let packet = self.codec.receive_timeout(&mut self.sock, self.clock.current_tick_end())?;
        if let Some(ref got_packet) = packet {
            match &got_packet {
                ServerPacket::ChunkData { chunk_x, chunk_z, .. } => {
                    trace!("Received: ChunkData {{ chunk_x: {}, chunk_z: {}, ... }}", chunk_x, chunk_z);
                }
                p => trace!("Received: {:?}", p)
            }
        } else {
            self.clock.advance();
        }
        Ok(packet)
    }

    pub fn health(&self) -> f32 {
        self.gamestate.health()
    }

    pub fn food(&self) -> f32 {
        self.gamestate.food()
    }

    pub fn my_position(&self) -> Position {
        self.gamestate.my_position()
    }

    pub fn say<M: Into<String>>(&mut self, msg: M) -> Result<()> {
        self.send(ClientPacket::ChatMessage { message: msg.into() })
    }

    fn send_position(&mut self) -> Result<()> {
        let position = self.gamestate.my_position();
        self.send(ClientPacket::PlayerPositionAndLook {
            x: position.x,
            y: position.y,
            z: position.z,
            yaw: self.gamestate.my_yaw(),
            pitch: 0f32,
            on_ground: true
        })
    }

    pub fn block_state_at(&self, position: BlockPosition) -> Option<BlockState> {
        self.gamestate.block_state_at(position)
    }

    pub fn find_block_ids_within(&self, block_id: u16, position: BlockPosition, distance: i32) -> Vec<BlockPosition> {
        self.gamestate.find_block_ids_within(block_id, position, distance)
    }

    pub fn find_path_to(&self, start: BlockPosition, dest: BlockPosition) -> Option<Vec<BlockPosition>> {
        self.gamestate.find_path_to(start, dest)
    }

    pub fn player_names(&self) -> Vec<&str> {
        self.gamestate.player_names()
    }

    pub fn current_tick(&self) -> i64 {
        self.clock.current_tick()
    }

    pub fn teleport_to(&mut self, position: Position) -> Result<()> {
        self.gamestate.teleport_to(position);
        self.send_position()
    }

    pub fn set_my_yaw(&mut self, angle: f32) -> Result<()> {
        self.gamestate.set_yaw(angle);
        self.send_position()
    }

    pub fn r#move(&mut self, flag: bool) {
        self.gamestate.r#move(flag);
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