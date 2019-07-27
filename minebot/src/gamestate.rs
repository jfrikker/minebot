use crate::geom::Orientation;

#[derive(Default)]
pub struct GameState {
    pub username: String,
    pub my_entity_id: EntityId,
    pub my_orientation: Orientation,
    pub health: f32,
    pub food: f32
}

type EntityId = i32;