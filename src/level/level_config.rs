use std::collections::HashMap;
use crate::level::generation::opening_weights::OpeningWeights;
use crate::level::room::kind::RoomKind;

pub struct LevelConfig {
    /// Maps each room type to its specific door generation weights
    pub room_weights: HashMap<RoomKind, OpeningWeights>,
}

impl LevelConfig {
    /// Example of a standard level distribution
    pub fn standard() -> Self {
        let mut room_weights = HashMap::new();
        room_weights.insert(RoomKind::Entrance, OpeningWeights::new([0, 10, 30, 40, 15, 5]));
        room_weights.insert(RoomKind::Normal, OpeningWeights::new([15, 45, 25, 10, 4, 1]));
        Self { room_weights }
    }

    /// Helper to grab weights for a specific kind
    pub fn get_weights(&self, kind: &RoomKind) -> &OpeningWeights {
        self.room_weights.get(kind).expect("Missing weights for RoomKind")
    }
}
