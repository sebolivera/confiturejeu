/// Config for level generation.
pub struct GenConfig {
    /// Rooms on the critical path (entrance → boss). Always hit exactly.
    pub main_path_len: usize,
    /// Dead-end side branches to sprout off the main path.
    pub branch_count: usize,
    /// Rooms per branch; each tip becomes a treasure room.
    pub branch_len: std::ops::RangeInclusive<usize>,
    /// Relative weight of a 0° / 60° / 120° / 180° turn from the current
    /// heading at each step. A heavy straight weight makes paths snake
    /// outward; flatter weights let them curl up into a blob.
    pub turn_weights: [u32; 4],
    /// Chance a step goes up/down instead of sideways.
    pub floor_change_chance: f32,
    /// Max floors.
    pub max_floors: u8,
}

impl Default for GenConfig {
    fn default() -> Self {
        Self {
            main_path_len: 100,
            branch_count: 10,
            branch_len: 3..=5,
            turn_weights: [8, 3, 1, 0],
            floor_change_chance: 0.1,
            max_floors: 3,
        }
    }
}

impl GenConfig {
    /// Constructor with full control.
    #[must_use]
    pub const fn new(
        main_path_len: usize,
        branch_count: usize,
        branch_len: std::ops::RangeInclusive<usize>,
        turn_weights: [u32; 4],
        floor_change_chance: f32,
        max_floors: u8,
    ) -> Self {
        Self {
            main_path_len,
            branch_count,
            branch_len,
            turn_weights,
            floor_change_chance,
            max_floors,
        }
    }
}
