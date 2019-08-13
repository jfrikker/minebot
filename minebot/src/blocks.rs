#[derive(Debug, Clone, Copy)]
pub struct BlockState(pub u16);

impl BlockState {
    pub fn id(&self) -> u16 {
        self.0 >> 4
    }

    pub fn meta(&self) -> u8 {
        (self.0 & 0x0F) as u8
    }

    pub fn is_passable(&self) -> bool {
        let id = self.id();
        id == 0 ||
            id == 31 ||
            id == 32
    }

    pub fn slipperiness(&self) -> f64 {
        match self.id() {
            0 => 0.91,
            _ => 0.6
        }
    }
}