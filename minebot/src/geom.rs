use libm::fmod;

pub type Distance = f64;
type Angle = f32;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Position {
    x: Distance,
    y: Distance,
    z: Distance
}

impl Position {
    pub fn new(x: Distance, y: Distance, z: Distance) -> Self {
        Position { x, y, z }
    }

    pub fn x(&self) -> Distance {
        self.x
    }

    pub fn set_x(&mut self, x: Distance) {
        self.x = x;
    }

    pub fn add_x(&mut self, x: Distance) {
        self.x += x;
    }

    pub fn y(&self) -> Distance {
        self.y
    }

    pub fn set_y(&mut self, y: Distance) {
        self.y = y;
    }

    pub fn add_y(&mut self, y: Distance) {
        self.y += y;
    }

    pub fn z(&self) -> Distance {
        self.z
    }

    pub fn set_z(&mut self, z: Distance) {
        self.z = z;
    }

    pub fn add_z(&mut self, z: Distance) {
        self.z += z;
    }

    pub fn chunk(&self) -> ChunkAddr {
        ChunkAddr::new((self.x / (CHUNK_WIDTH as f64)) as i32, (self.z / (CHUNK_WIDTH as f64)) as i32)
    }

    pub fn local(&self) -> LocalAddr {
        LocalAddr::new(fmod(self.x, CHUNK_WIDTH as f64) as u8, self.y as u8, fmod(self.z, CHUNK_WIDTH as f64) as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rotation {
    yaw: Angle,
    pitch: Angle
}

impl Rotation {
    pub fn new(yaw: Angle, pitch: Angle) -> Self {
        Rotation { yaw, pitch }
    }

    pub fn yaw(&self) -> Angle {
        self.yaw
    }

    pub fn set_yaw(&mut self, yaw: Angle) {
        self.yaw = yaw;
    }

    pub fn add_yaw(&mut self, yaw: Angle) {
        self.yaw += yaw;
    }

    pub fn pitch(&self) -> Angle {
        self.pitch
    }

    pub fn set_pitch(&mut self, pitch: Angle) {
        self.pitch = pitch;
    }

    pub fn add_pitch(&mut self, pitch: Angle) {
        self.pitch += pitch;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Orientation {
    position: Position,
    rotation: Rotation
}

impl Orientation {
    pub fn new(position: Position, rotation: Rotation) -> Self {
        Orientation { position, rotation }
    }

    pub fn from_parts(x: Distance, y: Distance, z: Distance, yaw: Angle, pitch: Angle) -> Self {
        Orientation::new(Position::new(x, y, z), Rotation::new(yaw, pitch))
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }

    pub fn rotation(&self) -> &Rotation {
        &self.rotation
    }

    pub fn rotation_mut(&mut self) -> &mut Rotation {
        &mut self.rotation
    }

    pub fn x(&self) -> Distance {
        self.position.x()
    }

    pub fn set_x(&mut self, x: Distance) {
        self.position.set_x(x);
    }

    pub fn add_x(&mut self, x: Distance) {
        self.position.add_x(x);
    }

    pub fn y(&self) -> Distance {
        self.position.y()
    }

    pub fn set_y(&mut self, y: Distance) {
        self.position.set_y(y);
    }

    pub fn add_y(&mut self, y: Distance) {
        self.position.add_y(y);
    }

    pub fn z(&self) -> Distance {
        self.position.z()
    }

    pub fn set_z(&mut self, z: Distance) {
        self.position.set_z(z);
    }

    pub fn add_z(&mut self, z: Distance) {
        self.position.add_z(z);
    }

    pub fn yaw(&self) -> Angle {
        self.rotation.yaw()
    }

    pub fn set_yaw(&mut self, yaw: Angle) {
        self.rotation.set_yaw(yaw);
    }

    pub fn add_yaw(&mut self, yaw: Angle) {
        self.rotation.add_yaw(yaw);
    }

    pub fn pitch(&self) -> Angle {
        self.rotation.pitch()
    }

    pub fn set_pitch(&mut self, pitch: Angle) {
        self.rotation.set_pitch(pitch);
    }

    pub fn add_pitch(&mut self, pitch: Angle) {
        self.rotation.add_pitch(pitch);
    }
}

pub const CHUNK_WIDTH: u8 = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalAddr(pub u16);

impl LocalAddr {
    pub fn new(x: u8, y: u8, z: u8) -> LocalAddr {
        LocalAddr((x as u16) | ((z as u16) << 4) | ((y as u16) << 8))
    }

    pub fn x(&self) -> u8 {
        (self.0 & 0x0F) as u8
    }

    pub fn y(&self) -> u8 {
        (self.0 >> 8 & 0xFF) as u8
    }

    pub fn z(&self) -> u8 {
        ((self.0 >> 4) & 0x0F) as u8
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ChunkAddr {
    x: i32,
    z: i32
}

impl ChunkAddr {
    pub fn new(x: i32, z: i32) -> ChunkAddr {
        ChunkAddr {
            x,
            z
        }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn z(&self) -> i32 {
        self.z
    }
}