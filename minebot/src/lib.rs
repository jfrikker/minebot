#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;

use nbt::{NbtDecode, NbtEncode};
use nbt::codec::NbtCodec;
use packets::*;
use std::fmt::Debug;
use std::net::TcpStream;

pub struct MinebotClient {
    sock: TcpStream,
    codec: NbtCodec,
    health: f32
}

impl MinebotClient {
    pub fn connect(host: String, port: u16, username: String) -> Result<Self> {
        info!("Connecting to {}:{}...", host, port);
        let sock = TcpStream::connect((&host as &str, port))?;
        let mut res = MinebotClient {
            sock,
            codec: NbtCodec::new(),
            health: 10.0
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

        let response: ServerLoginPacket = res.receive()?;

        match response {
            ServerLoginPacket::LoginSuccess {username: _, uuid} => info!("Successfully connected, player id is {}", uuid)
        }

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
        self.handle(&packet);
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

    fn handle(&mut self, packet: &ServerPacket) {
        match packet {
            ServerPacket::UpdateHealth { health, .. } => {
                self.health = health / 2.0
            }
            _ => {}
        };
    }

    fn receive<P: NbtDecode + Debug>(&mut self) -> Result<P> {
        let packet = self.codec.receive(&mut self.sock)?;
        trace!("Received: {:?}", packet);
        Ok(packet)
    }

    pub fn health(&self) -> f32 {
        self.health
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

type Result<T> = std::result::Result<T, Error>;