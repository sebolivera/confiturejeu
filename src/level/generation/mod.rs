/// Config for level generation.
pub mod gen_config;
/// Open weights.
pub mod opening_weights;

use rand::prelude::IndexedRandom;
use rand::Rng;
use crate::level::coordinates::Coordinates;
use crate::level::generation::gen_config::GenConfig;
use crate::level::{Direction, Level};
use crate::level::level_config::LevelConfig;
use crate::level::room::kind::RoomKind;
use crate::level::room::kind::RoomKind::Entrance;
use crate::level::room::Room;

/// Populates the level with rooms
pub fn generate(rng: &mut impl Rng, config: &GenConfig) -> Level {
    let mut level = Level::new();
    let level_config = LevelConfig::standard();
    let start_coords = Coordinates::new(0, 0, 0);
    let start_weights = level_config.get_weights(&Entrance);
    let start_openings_count = start_weights.sample_count(rng);

    let start_dirs = Direction::ALL.sample(rng, start_openings_count);
    let start_mask = start_dirs.fold(0, |mask, dir| mask | dir.mask());
    let mut current_coords = start_coords;
    let mut current_room = Room::new(Entrance, start_coords, start_mask);
    level.insert(current_room);
    for step in 1..config.main_path_len {
        let valid_exits: Vec<Direction> = current_room
            .open_directions()
            .filter(|&dir| {
                let neighbor_coords = current_coords.neighbor(dir);
                !level.rooms.contains_key(&neighbor_coords)
            })
            .collect();
        if valid_exits.is_empty() {
            break;
        }
        let exit_dir = *valid_exits.choose(rng).unwrap();
        let next_coords = current_coords.neighbor(exit_dir);
        let entry_dir = exit_dir.opposite();

        let is_boss = step == config.main_path_len - 1;
        let next_kind = if is_boss { RoomKind::Boss } else { RoomKind::Normal };

        let mut next_mask = entry_dir.mask();

        if next_kind != RoomKind::Boss {
            let next_weights = level_config.get_weights(&next_kind);
            let target_openings = next_weights.sample_count(rng);
            let additional_doors = target_openings.saturating_sub(1);

            let mut available_dirs = Direction::ALL.to_vec();
            available_dirs.retain(|&d| d != entry_dir);

            let extra_dirs = available_dirs.sample(rng, additional_doors);
            for &dir in extra_dirs {
                next_mask |= dir.mask();
            }
        }

        let next_room = Room::new(next_kind, next_coords, next_mask);
        level.insert(next_room);

        current_coords = next_coords;
        current_room = next_room;
    }

    level
}
