use bytes::{Buf, Bytes, IntoBuf};
use cgmath::{Vector3, vec3};
use cgmath::prelude::*;
use crate::geom::*;
use crate::blocks::BlockState;
use packets::{AddPlayer, PlayerListPacket, RemovePlayer, ServerPacket};
use pathfinding::directed::astar::astar;
use std::collections::HashMap;
use std::iter::{repeat, Cloned};
use uuid::Uuid;

pub struct GameState {
    players: HashMap<Uuid, Player>,
    my_id: Uuid,
    health: f32,
    food: f32,
    chunks: HashMap<ChunkAddr, Chunk>,
    entities: HashMap<EntityId, Entity>
}

impl GameState {
    pub fn new(my_id: Uuid, my_username: String) -> Self {
        let mut players = HashMap::default();
        players.insert(my_id, Player {
            name: my_username,
            entity_id: None
        });

        GameState {
            players,
            my_id,
            health: 10.0,
            food: 10.0,
            chunks: HashMap::default(),
            entities: HashMap::default()
        }
    }

    pub fn handle_packet(&mut self, packet: &ServerPacket) {
        match *packet {
            ServerPacket::BlockChange { position, block_state } => {
                let pos = BlockPosition::new((position >> 38) as i32, (position >> 26 & 0xFFF) as i32, (position & 0x3FFFFFF) as i32);
                let bs = BlockState(block_state as u16);
                self.set_block_state(pos, bs);
            }
            ServerPacket::ChunkData { chunk_x, chunk_z, full_chunk, primary_bitmask, ref data } => {
                if full_chunk {
                    self.load_chunk_data(chunk_x, chunk_z, primary_bitmask as u8, data)
                }
            }
            ServerPacket::JoinGame { entity_id, .. } => {
                self.players.get_mut(&self.my_id).unwrap().entity_id = Some(entity_id);
                self.entities.insert(entity_id, Entity::default());
            }
            ServerPacket::MultiBlockChange { chunk_x, chunk_z, ref records } => {
                let chunk_addr = ChunkAddr::new(chunk_x, chunk_z);
                for change in records.iter() {
                    let local_addr = from_local_index(change.local_addr);
                    let bs = BlockState(change.block_state as u16);
                    self.set_block_state(to_global_coords(chunk_addr, local_addr), bs);
                }
            }
            ServerPacket::PlayerList { packet: PlayerListPacket::AddPlayers { ref players } } => {
                for AddPlayer { uuid, name, .. } in players {
                    self.players.entry(uuid.clone()).or_insert_with(|| Player::with_name(name.into()));
                }
            }
            ServerPacket::PlayerList { packet: PlayerListPacket::RemovePlayers { ref players } } => {
                for RemovePlayer { uuid } in players {
                    self.players.remove(&uuid);
                }
            }
            ServerPacket::PlayerPositionAndLook {x, y, z, flags, .. } => {
                let my_position = &mut self.my_entity_mut().position;
                if flags & 0x01 != 0 {
                    my_position.x += x;
                } else {
                    my_position.x = x;
                }
                if flags & 0x02 != 0 {
                    my_position.y += y;
                } else {
                    my_position.y = y;
                }
                if flags & 0x04 != 0 {
                    my_position.z += z;
                } else {
                    my_position.z = z;
                }
                /*if flags & 0x08 != 0 {
                    my_orientation.add_yaw(yaw);
                } else {
                    my_orientation.set_yaw(yaw);
                }
                if flags & 0x10 != 0 {
                    my_orientation.add_pitch(pitch);
                } else {
                    my_orientation.set_pitch(pitch);
                }*/
            }
            ServerPacket::SpawnPlayer { uuid, entity_id, .. } => {
                self.entities.insert(entity_id, Entity::default());
                self.players.get_mut(&uuid).unwrap().set_entity_id(entity_id);
            }
            ServerPacket::UnloadChunk { chunk_x, chunk_z } => {
                self.unload_chunk(chunk_x, chunk_z);
            }
            ServerPacket::UpdateHealth { health, food, .. } => {
                self.health = health / 2.0;
                self.food = (food as f32) / 2.0;
            }
            _ => {}
        };

        if let Some(entity_id) = entity_id(packet) {
            self.handle_entity_packet(entity_id, packet);
        }
    }

    fn handle_entity_packet(&mut self, entity_id: EntityId, packet: &ServerPacket) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.handle_packet(packet);
        }
    }

    pub fn my_username(&self) -> &str {
        &self.players[&self.my_id].name
    }

    fn my_entity_id(&self) -> EntityId {
        self.players.get(&self.my_id).unwrap().entity_id.unwrap()
    }

    fn my_entity(&self) -> &Entity {
        self.entities.get(&self.my_entity_id()).unwrap()
    }

    fn my_entity_mut(&mut self) -> &mut Entity {
        self.entities.get_mut(&self.my_entity_id()).unwrap()
    }

    pub fn my_position(&self) -> &Position {
        &self.my_entity().position
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn food(&self) -> f32 {
        self.food
    }

    pub fn player_name(&self, id: &Uuid) -> Option<&str> {
        self.players.get(id).map(|p| p.name.as_ref())
    }

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

    pub fn player_names(&self) -> Vec<&str> {
        self.players.values()
            .map(|p| p.name.as_ref())
            .collect()
    }

    pub fn block_state_at(&self, position: BlockPosition) -> Option<BlockState> {
        let chunk = self.chunks.get(&chunk_addr(position))?;
        Some(chunk.block_state(local_addr(position)))
    }

    pub fn find_block_ids_within(&self, block_id: u16, position: BlockPosition, distance: i32) -> Vec<BlockPosition> {
        let min_pos = position + vec3(-distance, -distance, -distance);
        let max_pos = position + vec3(distance, distance, distance);
        let min_chunk = chunk_addr(min_pos);
        let max_chunk = chunk_addr(max_pos);
        let mut result = Vec::default();

        for chunk_x in min_chunk.x .. (max_chunk.x + 1) {
            for chunk_z in min_chunk.y .. (max_chunk.y + 1) {
                let chunk_addr = ChunkAddr::new(chunk_x, chunk_z);
                if let Some(chunk) = self.chunks.get(&chunk_addr) {
                    let matches = chunk.find_matching_block_state(|bs| bs.id() == block_id);
                    result.extend(
                        matches.into_iter()
                            .map(|pos| to_global_coords(chunk_addr, pos))
                            .filter(|pos| pos.x >= min_pos.x &&
                                pos.y >= min_pos.y &&
                                pos.z >= min_pos.z &&
                                pos.x <= max_pos.x &&
                                pos.y <= max_pos.y &&
                                pos.z <= max_pos.z)
                    );
                }
            }
        }

        result.sort_unstable_by(|pos1, pos2| {
            let pos1_dist = (pos1.cast::<f64>().unwrap() - position.cast::<f64>().unwrap()).magnitude2();
            let pos2_dist = (pos2.cast::<f64>().unwrap() - position.cast::<f64>().unwrap()).magnitude2();
            pos1_dist.partial_cmp(&pos2_dist).unwrap()
        });
        result
    }

    pub fn find_path_to(&self, start: BlockPosition, dest: BlockPosition) -> Option<Vec<BlockPosition>> {
        astar(&start, 
            |pos| self.find_walkable_positions(pos),
            |pos| (pos - dest).manhattan_length() + 1,
            |pos| (pos - dest).manhattan_length() < 2)
            .map(|(r, _)| r)
    }

    fn find_walkable_positions(&self, pos: &BlockPosition) -> Vec<(BlockPosition, i32)> {
        let mut result = Vec::default();
        let is_passable = |dir|
            self.block_state_at(pos + dir).map_or(false, |bs| bs.is_passable());

        let mut check_direction = |dir| {
            if is_passable(dir + Vector3::unit_y()) {
                if is_passable(dir) {
                    if !is_passable(dir - Vector3::unit_y()) {
                        result.push((pos + dir, 1));
                    } else if !is_passable(dir - 2 * Vector3::unit_y()) {
                        result.push((pos + dir - Vector3::unit_y(), 2));
                    }
                } else if is_passable(dir + 2 * Vector3::unit_y()) {
                        result.push((pos + dir + Vector3::unit_y(), 2));
                }
            }
        };
        
        check_direction(vec3(-1, 0, 0));
        check_direction(vec3(0, 0, -1));
        check_direction(vec3(1, 0, 0));
        check_direction(vec3(0, 0, 1));
        result
    }

    pub fn set_block_state(&mut self, pos: BlockPosition, state: BlockState) {
        let addr = chunk_addr(pos);
        if let Some(chunk) = self.chunks.get_mut(&addr) {
            chunk.set_block_state(local_addr(pos), state);
        } else {
            warn!("Block update received for unloaded chunk ({}, {})", addr.x, addr.y);
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
        chunk.set_block_state(from_local_index(addr + starting_idx), block_id);
        buf >>= bits_per_block;
        remaining -= bits_per_block;
    }

    for addr in 0..2048 {
        let temp = data.get_u8();
        chunk.set_light_level(from_local_index(2 * addr + starting_idx), temp & 0x0F);
        chunk.set_light_level(from_local_index(2 * addr + starting_idx + 1), temp >> 4);
    }

    for addr in 0..2048 {
        let temp = data.get_u8();
        chunk.set_skylight_level(from_local_index(2 * addr + starting_idx), temp & 0x0F);
        chunk.set_skylight_level(from_local_index(2 * addr + starting_idx + 1), temp >> 4);
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

    pub fn get(&self, addr: LocalAddr) -> T {
        self.data.get(to_local_index(addr) as usize).map_or(self.default, |val| *val)
    }

    pub fn set(&mut self, addr: LocalAddr, val: T) {
        let idx = to_local_index(addr) as usize;
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

    pub fn block_state(&self, addr: LocalAddr) -> BlockState {
        self.block_states.get(addr)
    }

    pub fn set_block_state(&mut self, addr: LocalAddr, val: BlockState) {
        self.block_states.set(addr, val);
    }

    pub fn damage(&self, addr: LocalAddr) -> u8 {
        self.damage.get(addr)
    }

    pub fn set_damage(&mut self, addr: LocalAddr, val: u8) {
        self.damage.set(addr, val);
    }

    pub fn light_level(&self, addr: LocalAddr) -> u8 {
        self.light.get(addr)
    }

    pub fn set_light_level(&mut self, addr: LocalAddr, val: u8) {
        self.light.set(addr, val);
    }

    pub fn skylight_level(&self, addr: LocalAddr) -> u8 {
        self.skylight.get(addr)
    }

    pub fn set_skylight_level(&mut self, addr: LocalAddr, val: u8) {
        self.skylight.set(addr, val);
    }

    pub fn find_matching_block_state(&self, pred: impl Fn(BlockState) -> bool) -> Vec<LocalAddr> {
        self.block_states.iter()
            .enumerate()
            .filter(|(_, bs)| pred(*bs))
            .map(|(idx, _)| from_local_index(idx as u16))
            .collect()
    }
}

fn read_varint(buf: &mut Buf) -> i32 {
    let mut result = 0;
    let mut read = 0;
    loop {
        let byte = buf.get_u8();
        result |= (byte as i32 & 0x7F) << (read * 7);

        if byte & 0x80 == 0 {
            return result;
        }
        read += 1;
    }
}

struct Player {
    name: String,
    entity_id: Option<EntityId>
}

impl Player {
    fn with_name(name: String) -> Self {
        Player {
            name,
            entity_id: None
        }
    }

    fn set_entity_id(&mut self, entity_id: EntityId) {
        self.entity_id = Some(entity_id);
    }
}

pub fn entity_id(packet: &ServerPacket) -> Option<EntityId> {
        match *packet {
            ServerPacket::SpawnPlayer { entity_id, ..} => Some(entity_id),
            _ => None
        }
}

struct Entity {
    position: Position
}

impl Default for Entity {
    fn default() -> Self {
        Entity {
            position: Position::origin()
        }
    }
}

impl Entity {
    pub fn handle_packet(&mut self, packet: &ServerPacket) {
        match *packet {
            ServerPacket::SpawnPlayer { x, y, z, ..} => {
                self.position = Position::new(x, y, z);
            }
            _ => ()
        }
    }
}