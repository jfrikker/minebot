use bytes::{Buf, Bytes, IntoBuf};
use crate::geom::{CHUNK_WIDTH, BlockPosition, ChunkAddr, LocalAddr, Orientation};
use crate::blocks::BlockState;
use pathfinding::directed::astar::astar;
use std::collections::HashMap;
use std::iter::{repeat, Cloned};
use uuid::Uuid;

#[derive(Default)]
pub struct GameState {
    pub username: String,
    pub players: HashMap<Uuid, String>,
    pub my_entity_id: EntityId,
    pub my_orientation: Orientation,
    pub health: f32,
    pub food: f32,
    chunks: HashMap<ChunkAddr, Chunk>
}

impl GameState {
    pub fn load_chunk_data(&mut self, chunk_x: i32, chunk_z: i32, 
        mut primary_bit_mask: u8, data: &Bytes) {
        trace!("Loading chunk at ({}, {})", chunk_x, chunk_z);
        let addr = ChunkAddr::new(chunk_x, chunk_z);
        let mut data = data.clone().into_buf();
        let mut chunk = Chunk::new();

        let mut y_offset: u8 = 0;
        while primary_bit_mask > 0 {
            if primary_bit_mask & 0x01 > 0 {
                load_single_chunk(&mut chunk, y_offset, &mut data);
            }
            y_offset += 16;
            primary_bit_mask >>= 1;
        }

        self.chunks.insert(addr, chunk);
    }

    pub fn unload_chunk(&mut self, chunk_x: i32, chunk_z: i32) {
        let addr = ChunkAddr::new(chunk_x, chunk_z);
        self.chunks.remove(&addr);
    }

    pub fn get_block_state_at(&self, position: &BlockPosition) -> Option<BlockState> {
        let chunk = self.chunks.get(&position.chunk())?;
        Some(chunk.get_block_state(&position.local()))
    }

    pub fn find_block_ids_within(&self, block_id: u16, position: &BlockPosition, distance: i32) -> Vec<BlockPosition> {
        let mut min_pos = position.clone();
        min_pos.add_x(-distance);
        min_pos.add_y(-distance);
        min_pos.add_z(-distance);

        let mut max_pos = position.clone();
        max_pos.add_x(distance);
        max_pos.add_y(distance);
        max_pos.add_z(distance);

        let min_chunk = min_pos.chunk();
        let max_chunk = max_pos.chunk();
        let mut result = Vec::default();

        for chunk_x in min_chunk.x() .. (max_chunk.x() + 1) {
            for chunk_z in min_chunk.z() .. (max_chunk.z() + 1) {
                let chunk_addr = ChunkAddr::new(chunk_x, chunk_z);
                if let Some(chunk) = self.chunks.get(&chunk_addr) {
                    let matches = chunk.find_matching_block_state(|bs| bs.get_id() == block_id);
                    result.extend(
                        matches.into_iter()
                            .map(|pos| BlockPosition::from_parts(chunk_addr, pos))
                            .filter(|pos| pos.x() >= min_pos.x() &&
                                pos.y() >= min_pos.y() &&
                                pos.z() >= min_pos.z() &&
                                pos.x() <= max_pos.x() &&
                                pos.y() <= max_pos.y() &&
                                pos.z() <= max_pos.z())
                    );
                }
            }
        }

        result.sort_unstable_by(|pos1, pos2| pos1.distance_to_ord(&position).partial_cmp(&pos2.distance_to_ord(&position)).unwrap());
        result
    }

    pub fn find_path_to(&self, start: BlockPosition, dest: BlockPosition) -> Option<Vec<BlockPosition>> {
        astar(&start, 
            |pos| self.find_walkable_positions(pos),
            |pos| (pos.distance_to(&dest) as u64) + 1,
            |pos| pos.distance_to(&dest) < 2.0)
            .map(|(r, _)| r)
    }

    fn find_walkable_positions(&self, pos: &BlockPosition) -> Vec<(BlockPosition, u64)> {
        let mut result = Vec::default();
        let is_passable = |x, y, z|
            self.get_block_state_at(&pos.with_diff(x, y, z)).map_or(false, |bs| bs.is_passable());

        let mut check_direction = |x, z| {
            if is_passable(x, 1, z) {
                if is_passable(x, 0, z) {
                    if !is_passable(x, -1, z) {
                        result.push((pos.with_diff(x, 0, z), 1));
                    } else if !is_passable(x, -2, z) {
                        result.push((pos.with_diff(x, -1, z), 2));
                    }
                } else if is_passable(x, 2, z) {
                        result.push((pos.with_diff(x, 1, z), 2));
                }
            }
        };
        
        check_direction(-1, 0);
        check_direction(0, -1);
        check_direction(1, 0);
        check_direction(0, 1);
        result
    }

    pub fn set_block_state(&mut self, pos: &BlockPosition, state: BlockState) {
        let addr = pos.chunk();
        if let Some(chunk) = self.chunks.get_mut(&addr) {
            chunk.set_block_state(&pos.local(), state);
        } else {
            warn!("Block update received for unloaded chunk ({}, {})", addr.x(), addr.z());
        }
    }
}

fn load_single_chunk(chunk: &mut Chunk, y_offset: u8, data: &mut Buf) {
    let bits_per_block = data.get_u8();
    let palette: Option<Vec<u16>> = if bits_per_block <= 8 {
        let palette_len = read_varint(data);
        let mut v = Vec::new();
        for _ in 0..palette_len {
            v.push(read_varint(data) as u16);
        }
        Some(v)
    } else {
        None
    };

    read_varint(data);

    let starting_idx: u16 = CHUNK_WIDTH as u16 * CHUNK_WIDTH as u16 * y_offset as u16;
    let mut buf: u128 = 0;
    let mut remaining: u8 = 0;
    for addr in 0..4096 {
        if remaining < bits_per_block {
            let temp = data.get_u64_be() as u128;
            buf |= temp << remaining;
            remaining += 64;
        }

        let temp_id: u16 = (buf & (0xFFFF >> (16 - bits_per_block as u16))) as u16;
        let block_id = BlockState(palette.as_ref().map_or(temp_id, |p| p[temp_id as usize]));
        chunk.set_block_state(&LocalAddr(addr + starting_idx), block_id);
        buf >>= bits_per_block;
        remaining -= bits_per_block;
    }

    for addr in 0..2048 {
        let temp = data.get_u8();
        chunk.set_light_level(&LocalAddr(2 * addr + starting_idx), temp & 0x0F);
        chunk.set_light_level(&LocalAddr(2 * addr + starting_idx + 1), temp >> 4);
    }

    for addr in 0..2048 {
        let temp = data.get_u8();
        chunk.set_skylight_level(&LocalAddr(2 * addr + starting_idx), temp & 0x0F);
        chunk.set_skylight_level(&LocalAddr(2 * addr + starting_idx + 1), temp >> 4);
    }
}

type EntityId = i32;

struct PerBlock<T: Copy> {
    data: Vec<T>,
    default: T
}

impl <T: Copy> PerBlock<T> {
    pub fn new(default: T) -> PerBlock<T> {
        let mut res = PerBlock {
            data: Vec::new(),
            default
        };
        res.data.reserve(80 as usize * CHUNK_WIDTH as usize * CHUNK_WIDTH as usize);
        res
    }

    pub fn get(&self, &LocalAddr(addr): &LocalAddr) -> T {
        self.data.get(addr as usize).map_or(self.default, |val| *val)
    }

    pub fn set(&mut self, &LocalAddr(addr): &LocalAddr, val: T) {
        let idx = addr as usize;
        if self.data.len() <= idx {
            let to_extend = idx - self.data.len() + 1;
            self.data.extend(repeat(self.default).take(to_extend));
        }
        self.data[idx] = val;
    }

    pub fn iter(&self) -> Cloned<std::slice::Iter<T>> {
        self.data.iter().cloned()
    }

    /*pub fn trim(&mut self) {
        let mut new_len = self.data.len();
        while (new_len > 0 && self.data[new_len - 1] == self.default) {
            new_len -= 1;
        }
        self.data.truncate(new_len);
    }*/
}

pub struct Chunk {
    block_states: PerBlock<BlockState>,
    damage: PerBlock<u8>,
    light: PerBlock<u8>,
    skylight: PerBlock<u8>
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            block_states: PerBlock::new(BlockState(0)),
            damage: PerBlock::new(0),
            light: PerBlock::new(0),
            skylight: PerBlock::new(15)
        }
    }

    pub fn get_block_state(&self, addr: &LocalAddr) -> BlockState {
        self.block_states.get(addr)
    }

    pub fn set_block_state(&mut self, addr: &LocalAddr, val: BlockState) {
        self.block_states.set(addr, val);
    }

    pub fn get_damage(&self, addr: &LocalAddr) -> u8 {
        self.damage.get(addr)
    }

    pub fn set_damage(&mut self, addr: &LocalAddr, val: u8) {
        self.damage.set(addr, val);
    }

    pub fn get_light_level(&self, addr: &LocalAddr) -> u8 {
        self.light.get(addr)
    }

    pub fn set_light_level(&mut self, addr: &LocalAddr, val: u8) {
        self.light.set(addr, val);
    }

    pub fn get_skylight_level(&self, addr: &LocalAddr) -> u8 {
        self.skylight.get(addr)
    }

    pub fn set_skylight_level(&mut self, addr: &LocalAddr, val: u8) {
        self.skylight.set(addr, val);
    }

    pub fn find_matching_block_state(&self, pred: impl Fn(BlockState) -> bool) -> Vec<LocalAddr> {
        self.block_states.iter()
            .enumerate()
            .filter(|(_, bs)| pred(*bs))
            .map(|(idx, _)| LocalAddr(idx as u16))
            .collect()
    }
}

fn read_varint(buf: &mut Buf) -> i32 {
    let mut result = 0;
    let mut read = 0;
    loop {
        let byte = buf.get_u8();
        result = result | ((byte as i32 & 0x7F) << (read * 7));

        if byte & 0x80 == 0 {
            return result;
        }
        read += 1;
    }
}