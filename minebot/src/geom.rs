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