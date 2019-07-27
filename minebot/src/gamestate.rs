use bytes::{Buf, Bytes, IntoBuf};
use crate::geom::{CHUNK_WIDTH, ChunkAddr, LocalAddr, Orientation};
use std::collections::HashMap;
use std::iter::repeat;

#[derive(Default)]
pub struct GameState {
    pub username: String,
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

    let mut buf: u128 = 0;
    let mut remaining: u8 = 0;
    for addr in 0..4096 {
        if remaining < bits_per_block {
            let temp = data.get_u64_be() as u128;
            buf |= temp << remaining;
            remaining += 64;
        }

        let temp_id = (buf & (0xFFFFFFFFFFFFFFFFFFFFFFFF >> (128 - bits_per_block as u16))) as u16;
        let block_id = palette.as_ref().map_or(temp_id, |p| p[temp_id as usize]);
        chunk.set_block_id(&LocalAddr(addr + (CHUNK_WIDTH as u16 * CHUNK_WIDTH as u16 * y_offset as u16)), block_id);
        buf >>= bits_per_block;
        remaining -= bits_per_block;
    }

    for addr in 0..2048 {
        let temp = data.get_u8();
        chunk.set_light_level(&LocalAddr(addr + (CHUNK_WIDTH as u16 * CHUNK_WIDTH as u16 * y_offset as u16)), temp & 0x0F);
        chunk.set_light_level(&LocalAddr(addr + (CHUNK_WIDTH as u16 * CHUNK_WIDTH as u16 * y_offset as u16) + 1), temp >> 4);
    }

    for addr in 0..2048 {
        let temp = data.get_u8();
        chunk.set_skylight_level(&LocalAddr(addr + (CHUNK_WIDTH as u16 * CHUNK_WIDTH as u16 * y_offset as u16)), temp & 0x0F);
        chunk.set_skylight_level(&LocalAddr(addr + (CHUNK_WIDTH as u16 * CHUNK_WIDTH as u16 * y_offset as u16) + 1), temp >> 4);
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

    /*pub fn trim(&mut self) {
        let mut new_len = self.data.len();
        while (new_len > 0 && self.data[new_len - 1] == self.default) {
            new_len -= 1;
        }
        self.data.truncate(new_len);
    }*/
}

pub struct Chunk {
    block_ids: PerBlock<u16>,
    damage: PerBlock<u8>,
    light: PerBlock<u8>,
    skylight: PerBlock<u8>
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            block_ids: PerBlock::new(0),
            damage: PerBlock::new(0),
            light: PerBlock::new(0),
            skylight: PerBlock::new(15)
        }
    }

    pub fn get_block_id(&self, addr: &LocalAddr) -> u16 {
        self.block_ids.get(addr)
    }

    pub fn set_block_id(&mut self, addr: &LocalAddr, val: u16) {
        self.block_ids.set(addr, val);
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