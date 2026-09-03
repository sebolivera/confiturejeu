/// Config for level generation.
pub struct GenConfig {
    /// Rooms on the critical path (entrance → boss).
    pub main_path_len: usize,
    /// How many side branches to sprout.
    pub branch_count: usize,
    /// Rooms per branch.
    pub branch_len: std::ops::RangeInclusive<usize>,
    /// Chance a step goes up/down instead of sideways.
    pub floor_change_chance: f32,
    /// Max floors.
    pub max_floors: u8,
}

impl Default for GenConfig {
    fn default() -> Self {
        Self {
            main_path_len: 10,
            branch_count: 2,
            branch_len: 3..=5,
            floor_change_chance: 0.1,
            max_floors: 3,
        }
    }
}

impl GenConfig {
    /// Constructor with full control.
    fn new(
        main_path_len: usize,
        branch_count: usize,
        branch_len: std::ops::RangeInclusive<usize>,
        floor_change_chance: f32,
        max_floors: u8,
    ) -> Self {
        Self {
            main_path_len,
            branch_count,
            branch_len,
            floor_change_chance,
            max_floors,
        }
    }
}
