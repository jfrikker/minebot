#[derive(Debug, Clone, Copy)]
pub struct BlockState(pub u16);

impl BlockState {
    pub fn get_id(&self) -> u16 {
        self.0 >> 4
    }

    pub fn get_meta(&self) -> u8 {
        (self.0 & 0x0F) as u8
    }

    pub fn is_passable(&self) -> bool {
        let id = self.get_id();
        id == 0 ||
            id == 31 ||
            id == 32
    }
}