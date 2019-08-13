use cgmath::{Point2, Point3, Vector3};
use cgmath::BaseNum;
use cgmath::num_traits::sign::Signed;
use cgmath::prelude::*;
use divrem::{DivFloor, RemFloor};

pub type Position = Point3<f64>;
pub type BlockPosition = Point3<i32>;
pub type Velocity = Vector3<f64>;

pub fn to_block_position(position: Position) -> BlockPosition {
    position.map(|c| c.floor() as i32)
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rotation {
    yaw: f64,
    pitch: f64
}

impl Rotation {
    pub fn new(yaw: f64, pitch: f64) -> Self {
        Rotation { yaw, pitch }
    }

    pub fn yaw(self) -> f64 {
        self.yaw
    }

    pub fn set_yaw(&mut self, yaw: f64) {
        self.yaw = yaw;
    }

    pub fn add_yaw(&mut self, yaw: f64) {
        self.yaw += yaw;
    }

    pub fn pitch(self) -> f64 {
        self.pitch
    }

    pub fn set_pitch(&mut self, pitch: f64) {
        self.pitch = pitch;
    }

    pub fn add_pitch(&mut self, pitch: f64) {
        self.pitch += pitch;
    }
}

pub fn to_global_coords(chunk: ChunkAddr, local: LocalAddr) -> BlockPosition {
    let temp = Point3 {
        x: chunk[0],
        y: 0,
        z: chunk[1]
    };
    temp * (CHUNK_WIDTH as i32) + (local.to_vec().cast::<i32>().unwrap())
}

pub fn chunk_addr(addr: BlockPosition) -> ChunkAddr {
    ChunkAddr {
        x: addr.x.div_floor(CHUNK_WIDTH as i32),
        y: addr.z.div_floor(CHUNK_WIDTH as i32)
    }
}

pub fn local_addr(addr: BlockPosition) -> LocalAddr {
    LocalAddr {
        x: addr.x.rem_floor(CHUNK_WIDTH as i32) as u8,
        y: addr.y as u8,
        z: addr.z.rem_floor(CHUNK_WIDTH as i32) as u8
    }
}

pub const CHUNK_WIDTH: u8 = 16;

pub type LocalAddr = Point3<u8>;
pub type ChunkAddr = Point2<i32>;

pub fn from_local_index(idx: u16) -> LocalAddr {
    LocalAddr {
        x: (idx & 0x0F) as u8,
        y: ((idx >> 8) & 0xFF) as u8,
        z: ((idx >> 4) & 0x0F) as u8
    }
}

pub fn to_local_index(addr: LocalAddr) -> u16 {
    (addr.x as u16) | ((addr.y as u16) << 8) | ((addr.z as u16) << 4)
}

pub trait ManhattanLength {
    type Scalar;

    fn manhattan_length(self) -> Self::Scalar;
}

impl <S: BaseNum + Signed> ManhattanLength for Vector3<S> {
    type Scalar = S;

    fn manhattan_length(self) -> S {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
}