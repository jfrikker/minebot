#[macro_use]
extern crate nbt_derive;

use bytes::Bytes;
use bytes::buf::BufMut;
use nbt::{NbtDecode, NbtEncode, NbtString};

pub type Angle = u8;

#[derive(Debug, NbtEncode)]
pub enum HandshakePacket {
    #[nbt(ordinal = "0")]
    HandshakePacket {
        #[nbt(codec = "varnum")] version: i32,
        host: String,
        port: u16,
        #[nbt(codec = "varnum")] next: i32
    }
}

#[derive(Debug, NbtEncode)]
pub enum ClientLoginPacket {
    #[nbt(ordinal = "0")]
    LoginStart {
        name: String
    }
}

#[derive(Debug, NbtDecode)]
pub enum ServerLoginPacket {
    #[nbt(ordinal = "2")]
    LoginSuccess {
        uuid: NbtString,
        username: NbtString
    }
}

#[derive(NbtDecode)]
pub enum ClientPacket {
    #[nbt(ordinal = "-1")]
    Fake
}

#[derive(Debug, NbtDecode)]
pub enum ServerPacket {
    #[nbt(ordinal = "0")]
    SpawnObject {
        // TODO: Rest
    },
    #[nbt(ordinal = "1")]
    SpawnExperienceOrb {
        // TODO: Rest
    },
    #[nbt(ordinal = "2")]
    SpawnGlobalEntity {
        // TODO: Rest
    },
    #[nbt(ordinal = "3")]
    SpawnMob {
        // TODO: Rest
    },
    #[nbt(ordinal = "4")]
    SpawnPainting {
        // TODO: Rest
    },
    #[nbt(ordinal = "5")]
    SpawnPlayer {
        // TODO: Rest
    },
    #[nbt(ordinal = "6")]
    Animation {
        // TODO: Rest
    },
    #[nbt(ordinal = "7")]
    Statistics {
        // TODO: Rest
    },
    #[nbt(ordinal = "8")]
    BlockBreakAnimation {
        // TODO: Rest
    },
    #[nbt(ordinal = "9")]
    UpdateBlockEntity {
        // TODO: Rest
    },
    #[nbt(ordinal = "10")]
    BlockAction {
        // TODO: Rest
    },
    #[nbt(ordinal = "11")]
    BlockChange {
        // TODO: Rest
    },
    #[nbt(ordinal = "12")]
    BossBar {
        // TODO: Rest
    },
    #[nbt(ordinal = "13")]
    ServerDifficulty {
        difficulty: Difficulty
    },
    #[nbt(ordinal = "14")]
    TabComplete {
        // TODO: Rest
    },
    #[nbt(ordinal = "15")]
    ChatMessage {
        // TODO: Rest
    },
    #[nbt(ordinal = "16")]
    MultiBlockChange {
        // TODO: Rest
    },
    #[nbt(ordinal = "17")]
    ConfirmTransaction {
        // TODO: Rest
    },
    #[nbt(ordinal = "18")]
    CloseWindow {
        // TODO: Rest
    },
    #[nbt(ordinal = "19")]
    OpenWindow {
        // TODO: Rest
    },
    #[nbt(ordinal = "20")]
    WindowItems {
        // TODO: Rest
    },
    #[nbt(ordinal = "21")]
    WindowProperty {
        // TODO: Rest
    },
    #[nbt(ordinal = "22")]
    SetSlot {
        // TODO: Rest
    },
    #[nbt(ordinal = "23")]
    SetCooldown {
        // TODO: Rest
    },
    #[nbt(ordinal = "24")]
    PluginMessage {
        channel: NbtString
        // TODO: data
    },
    #[nbt(ordinal = "25")]
    SoundEffect {
        // TODO: Rest
    },
    #[nbt(ordinal = "26")]
    Disconnect {
        // TODO: Rest
    },
    #[nbt(ordinal = "27")]
    EntityStatus {
        entity_id: i32,
        status: u8
    },
    #[nbt(ordinal = "28")]
    Explosion {
        // TODO: Rest
    },
    #[nbt(ordinal = "29")]
    UnloadChunk {
        // TODO: Rest
    },
    #[nbt(ordinal = "30")]
    ChangeGameState {
        // TODO: Rest
    },
    #[nbt(ordinal = "31")]
    KeepAlive {
        // TODO: Rest
    },
    #[nbt(ordinal = "32")]
    ChunkData {
        // TODO: Rest
    },
    #[nbt(ordinal = "33")]
    Effect {
        // TODO: Rest
    },
    #[nbt(ordinal = "34")]
    Particle {
        // TODO: Rest
    },
    #[nbt(ordinal = "35")]
    JoinGame {
        entity_id: i32,
        game_mode: FullGameMode,
        dimension: DimensionId,
        difficulty: Difficulty,
        max_players: u8,
        level_type: NbtString,
        reduced_debug_info: bool
    },
    #[nbt(ordinal = "36")]
    Map {
        // TODO: Rest
    },
    #[nbt(ordinal = "37")]
    Entity {
        // TODO: Rest
    },
    #[nbt(ordinal = "38")]
    EntityRelativeMove {
        #[nbt(codec = "varnum")] entity_id: i32,
        delta_x: i16,
        delta_y: i16,
        delta_z: i16,
        on_ground: bool
    },
    #[nbt(ordinal = "39")]
    EntityLookAndRelativeMove {
        // TODO: Rest
    },
    #[nbt(ordinal = "40")]
    EntityLook {
        #[nbt(codec = "varnum")] entity_id: i32,
        yaw: Angle,
        pitch: Angle,
        on_ground: bool
    },
    #[nbt(ordinal = "41")]
    VehicleMove {
        // TODO: Rest
    },
    #[nbt(ordinal = "42")]
    OpenSignEditor {
        // TODO: Rest
    },
    #[nbt(ordinal = "43")]
    CraftRecipeResponse {
        // TODO: Rest
    },
    #[nbt(ordinal = "44")]
    PlayerAbilities {
        flags: u8,
        flying_speed: f32,
        fov: f32
    },
    #[nbt(ordinal = "45")]
    CombatEvent {
        // TODO: Rest
    },
    #[nbt(ordinal = "46")]
    PlayerList {
        // TODO: Rest
    },
    #[nbt(ordinal = "47")]
    PlayerPositionAndLook {
        // TODO: Rest
    },
    #[nbt(ordinal = "48")]
    UseBed {
        // TODO: Rest
    },
    #[nbt(ordinal = "49")]
    UnlockRecipes {
        #[nbt(codec = "varnum")] action: i32,
        book_open: bool,
        filtering: bool
        // TODO: Rest
    },
    #[nbt(ordinal = "50")]
    DestroyEntities {
        // TODO: Rest
    },
    #[nbt(ordinal = "51")]
    RemoveEntityEffect {
        // TODO: Rest
    },
    #[nbt(ordinal = "52")]
    ResourcePackSend {
        // TODO: Rest
    },
    #[nbt(ordinal = "53")]
    Respawn {
        // TODO: Rest
    },
    #[nbt(ordinal = "54")]
    EntityHeadLook {
        #[nbt(codec = "varnum")] entity_id: i32,
        yaw: Angle
    },
    #[nbt(ordinal = "55")]
    SelectAdvancementTab {
        // TODO: Rest
    },
    #[nbt(ordinal = "56")]
    WorldBorder {
        // TODO: Rest
    },
    #[nbt(ordinal = "57")]
    Camera {
        // TODO: Rest
    },
    #[nbt(ordinal = "58")]
    HeldItemChange {
        slot: u8
    },
    #[nbt(ordinal = "59")]
    DisplayScoreboard {
        // TODO: Rest
    },
    #[nbt(ordinal = "60")]
    EntityMetadata {
        // TODO: Rest
    },
    #[nbt(ordinal = "61")]
    AttachEntity {
        // TODO: Rest
    },
    #[nbt(ordinal = "62")]
    EntityVelocity {
        #[nbt(codec = "varnum")] entity_id: i32,
        velocity_x: i16,
        velocity_y: i16,
        velocity_z: i16
    },
    #[nbt(ordinal = "63")]
    EntityEquipment {
        // TODO: Rest
    },
    #[nbt(ordinal = "64")]
    SetExperience {
        // TODO: Rest
    },
    #[nbt(ordinal = "65")]
    UpdateHealth {
        health: f32,
        #[nbt(codec = "varnum")] food: i32,
        saturation: f32
    },
    #[nbt(ordinal = "66")]
    ScoreboardObjective {
        // TODO: Rest
    },
    #[nbt(ordinal = "67")]
    SetPassengers {
        // TODO: Rest
    },
    #[nbt(ordinal = "68")]
    Teams {
        // TODO: Rest
    },
    #[nbt(ordinal = "69")]
    UpdateScore {
        // TODO: Rest
    },
    #[nbt(ordinal = "70")]
    SpawnPosition {
        // TODO: Rest
    },
    #[nbt(ordinal = "71")]
    TimeUpdate {
        world_age: i64,
        time_of_day: i64
    },
    #[nbt(ordinal = "72")]
    Title {
        // TODO: Rest
    },
    #[nbt(ordinal = "73")]
    SoundEffect2 {
        // TODO: Rest
    },
    #[nbt(ordinal = "74")]
    PlayerListHeaderFooter {
        // TODO: Rest
    },
    #[nbt(ordinal = "75")]
    CollectItem {
        // TODO: Rest
    },
    #[nbt(ordinal = "76")]
    EntityTeleport {
        // TODO: Rest
    },
    #[nbt(ordinal = "77")]
    Advancements {
        // TODO: Rest
    },
    #[nbt(ordinal = "78")]
    EntityProperties {
        // TODO: Rest
    },
    #[nbt(ordinal = "79")]
    EntityEffect {
        // TODO: Rest
    },
}

#[derive(Debug, Clone, Copy)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator
}

#[derive(Debug, Clone, Copy)]
pub struct FullGameMode {
    pub mode: GameMode,
    pub hardcore: bool
}

impl NbtDecode for FullGameMode {
    fn decode(buf: &mut Bytes) -> Self {
        let code = u8::decode(buf);
        let mode = match code & 0x0F {
            0 => GameMode::Survival,
            1 => GameMode::Creative,
            2 => GameMode::Adventure,
            3 => GameMode::Spectator,
            _ => panic!("Unexpected code {}", code)
        };
        let hardcore = code & 0xF0 > 0;

        FullGameMode {
            mode,
            hardcore
        }
    }
}

#[derive(Debug)]
pub enum DimensionId {
    Nether,
    Overworld,
    End
}

impl NbtDecode for DimensionId {
    fn decode(buf: &mut Bytes) -> Self {
        match i32::decode(buf) {
            -1 => DimensionId::Nether,
            0 => DimensionId::Overworld,
            1 => DimensionId::End,
            d => panic!("Unexpected dimension {}", d)
        }
    }
}

#[derive(Debug, NbtDecode)]
pub enum Difficulty {
    #[nbt(ordinal = "0")] Peaceful,
    #[nbt(ordinal = "1")] Easy,
    #[nbt(ordinal = "2")] Normal,
    #[nbt(ordinal = "3")] Hard
}