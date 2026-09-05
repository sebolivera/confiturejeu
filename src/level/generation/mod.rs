/// Config for level generation.
pub mod gen_config;

use crate::level::coordinates::Coordinates;
use crate::level::generation::gen_config::GenConfig;
use crate::level::room::Room;
use crate::level::room::kind::RoomKind;
use crate::level::{Direction, Level};
use rand::prelude::IndexedRandom;
use rand::{Rng, RngExt};

/// How many times a walk may dead-end before we give up on the config.
const MAX_ATTEMPTS: usize = 1000;

/// Populates the level with rooms: a self-avoiding path from the entrance to
/// the boss, plus dead-end side branches capped with treasure rooms.
///
/// The main path always has exactly `config.main_path_len` rooms: a walk that
/// traps itself before reaching that length is thrown away and retried. Steps
/// are weighted by [`GenConfig::turn_weights`] so paths tend to snake outward
/// instead of curling into a blob. Every door leads to an existing room and is
/// open from both sides.
///
/// # Panics
///
/// Panics if no walk reaches `config.main_path_len` rooms after
/// [`MAX_ATTEMPTS`] tries, which points at an infeasible config
/// (e.g. all-zero `turn_weights`).
pub fn generate(rng: &mut impl Rng, config: &GenConfig) -> Level {
    for _ in 0..MAX_ATTEMPTS {
        if let Some((mut level, path)) = try_generate(rng, config) {
            grow_branches(rng, config, &mut level, &path);
            return level;
        }
    }
    panic!(
        "no layout with a {}-room main path after {MAX_ATTEMPTS} attempts",
        config.main_path_len
    );
}

/// Weight of stepping towards `dir` when the walk entered the current room
/// from `entry`. The entrance has no heading, so its first step is uniform.
fn turn_weight(entry: Option<Direction>, dir: Direction, weights: &[u32; 4]) -> u32 {
    entry.map_or(1, |entry| {
        let heading = entry.opposite();
        let diff = (dir as i32 - heading as i32).rem_euclid(6) as usize;
        weights[diff.min(6 - diff)]
    })
}

/// Free tiles around `target`, ignoring `from` (the tile we would come from).
///
/// Turn weights only shape *local* curvature; globally a walk still wanders
/// back and then slides along its own body, packing rooms into a cluster.
/// Scaling each step by the elbow room at its destination steers walks away
/// from occupied territory — and away from pockets they could get trapped in.
fn elbow_room(level: &Level, from: Coordinates, target: Coordinates) -> u32 {
    Direction::ALL
        .into_iter()
        .map(|dir| target.neighbor(dir))
        .filter(|&tile| tile != from && !level.rooms.contains_key(&tile))
        .count() as u32
}

/// Full weight of a step: turn preference times destination elbow room.
fn step_weight(
    level: &Level,
    from: Coordinates,
    entry: Option<Direction>,
    dir: Direction,
    weights: &[u32; 4],
) -> u32 {
    turn_weight(entry, dir, weights) * elbow_room(level, from, from.neighbor(dir))
}

/// One self-avoiding walk of exactly `config.main_path_len` rooms.
///
/// Also returns the path coordinates in walk order, minus the boss room:
/// the anchors branches may sprout from. Returns [`None`] when the walk
/// dead-ends (every neighbour tile of the current room is occupied) so the
/// caller can retry.
fn try_generate(rng: &mut impl Rng, config: &GenConfig) -> Option<(Level, Vec<Coordinates>)> {
    let mut level = Level::new();
    let mut path = Vec::with_capacity(config.main_path_len);
    let mut coords = Coordinates::new(0, 0, 0);
    // The side the current room was entered from; the entrance has none.
    let mut entry: Option<Direction> = None;

    for step in 0..config.main_path_len {
        let is_last = step + 1 == config.main_path_len;
        let kind = if step == 0 {
            RoomKind::Entrance
        } else if is_last {
            RoomKind::Boss
        } else {
            RoomKind::Normal
        };

        let mut mask = entry.map_or(0, Direction::mask);

        if is_last {
            // The boss room only gets its entry door.
            level.insert(Room::new(kind, coords, mask));
            break;
        }

        let free_exits: Vec<Direction> = Direction::ALL
            .into_iter()
            .filter(|&dir| !level.rooms.contains_key(&coords.neighbor(dir)))
            .collect();
        let exit = *free_exits
            .choose_weighted(rng, |&dir| {
                step_weight(&level, coords, entry, dir, &config.turn_weights)
            })
            .ok()?;
        mask |= exit.mask();

        level.insert(Room::new(kind, coords, mask));
        path.push(coords);
        coords = coords.neighbor(exit);
        entry = Some(exit.opposite());
    }

    Some((level, path))
}

/// Grows dead-end side branches off the main path.
///
/// Each branch anchors at a random main-path room (never the boss: its lone
/// door stays the dramatic one), walks a weighted self-avoiding path and caps
/// its tip with a treasure room. A branch that dead-ends early stays short;
/// when no anchor has a free side left, branching stops.
fn grow_branches(
    rng: &mut impl Rng,
    config: &GenConfig,
    level: &mut Level,
    anchors: &[Coordinates],
) {
    for _ in 0..config.branch_count {
        let has_free_side = |coords: Coordinates| {
            Direction::ALL
                .into_iter()
                .any(|dir| !level.rooms.contains_key(&coords.neighbor(dir)))
        };
        let open_anchors: Vec<Coordinates> = anchors
            .iter()
            .copied()
            .filter(|&c| has_free_side(c))
            .collect();
        let Some(&start) = open_anchors.choose(rng) else {
            break;
        };

        let target_len = rng.random_range(config.branch_len.clone());
        let mut coords = start;
        // No heading yet: the first step off the main path is uniform.
        let mut entry: Option<Direction> = None;
        let mut tip = None;

        for _ in 0..target_len {
            let free_exits: Vec<Direction> = Direction::ALL
                .into_iter()
                .filter(|&dir| !level.rooms.contains_key(&coords.neighbor(dir)))
                .collect();
            let Ok(&exit) = free_exits.choose_weighted(rng, |&dir| {
                step_weight(level, coords, entry, dir, &config.turn_weights)
            }) else {
                break;
            };

            if let Some(room) = level.rooms.get_mut(&coords) {
                room.openings |= exit.mask();
            }
            coords = coords.neighbor(exit);
            entry = Some(exit.opposite());
            level.insert(Room::new(RoomKind::Normal, coords, exit.opposite().mask()));
            tip = Some(coords);
        }

        if let Some(room) = tip.and_then(|coords| level.rooms.get_mut(&coords)) {
            room.kind = RoomKind::Treasure;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GenConfig, generate, turn_weight};
    use crate::level::room::kind::RoomKind;
    use crate::level::{Direction, Level};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn levels() -> impl Iterator<Item = (u64, Level)> {
        (0..50).map(|seed| {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            (seed, generate(&mut rng, &GenConfig::default()))
        })
    }

    #[test]
    fn the_main_path_is_never_truncated() {
        let config = GenConfig::default();
        let max = config.main_path_len + config.branch_count * config.branch_len.end();
        for (seed, level) in levels() {
            let rooms = level.rooms.len();
            assert!(
                (config.main_path_len..=max).contains(&rooms),
                "seed {seed}: {rooms} rooms"
            );
            let bosses = level
                .rooms
                .values()
                .filter(|r| r.kind == RoomKind::Boss)
                .count();
            assert_eq!(bosses, 1, "seed {seed} has {bosses} boss rooms");
        }
    }

    #[test]
    fn branches_sprout_and_end_in_a_treasure_dead_end() {
        for (seed, level) in levels() {
            let treasures: Vec<_> = level
                .rooms
                .values()
                .filter(|r| r.kind == RoomKind::Treasure)
                .collect();
            assert!(!treasures.is_empty(), "seed {seed} grew no branch at all");
            for room in treasures {
                assert_eq!(
                    room.open_directions().count(),
                    1,
                    "seed {seed}: treasure at {:?} is not a dead end",
                    room.coords
                );
            }
        }
    }

    #[test]
    fn no_branches_means_exactly_the_main_path() {
        let config = GenConfig {
            branch_count: 0,
            ..GenConfig::default()
        };
        for seed in 0..10 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let level = generate(&mut rng, &config);
            assert_eq!(level.rooms.len(), config.main_path_len, "seed {seed}");
        }
    }

    #[test]
    fn every_door_leads_to_a_room_that_opens_back() {
        for (seed, level) in levels() {
            for room in level.rooms.values() {
                for dir in room.open_directions() {
                    let neighbor = level.rooms.get(&room.coords.neighbor(dir));
                    assert!(
                        neighbor.is_some_and(|n| n.is_open(dir.opposite())),
                        "seed {seed}: {:?} opens {dir:?} onto nothing",
                        room.coords
                    );
                }
            }
        }
    }

    #[test]
    fn every_room_is_reachable_from_the_entrance() {
        for (seed, level) in levels() {
            let entrance = level
                .rooms
                .values()
                .find(|r| r.kind == RoomKind::Entrance)
                .expect("no entrance")
                .coords;

            let mut seen = std::collections::HashSet::from([entrance]);
            let mut queue = vec![entrance];
            while let Some(coords) = queue.pop() {
                for next in level.rooms[&coords].accessible_neighbors() {
                    if seen.insert(next) {
                        queue.push(next);
                    }
                }
            }

            assert_eq!(
                seen.len(),
                level.rooms.len(),
                "seed {seed}: some rooms are unreachable"
            );
        }
    }

    #[test]
    fn a_single_room_level_is_just_the_entrance() {
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let config = GenConfig {
            main_path_len: 1,
            branch_count: 0,
            ..GenConfig::default()
        };
        let level = generate(&mut rng, &config);

        assert_eq!(level.rooms.len(), 1);
        let room = level.rooms.values().next().unwrap();
        assert_eq!(room.kind, RoomKind::Entrance);
        assert!(Direction::ALL.into_iter().all(|dir| !room.is_open(dir)));
    }

    #[test]
    fn turn_weights_are_indexed_by_turn_magnitude() {
        let weights = [8, 3, 1, 0];
        // Heading East (i.e. we entered the room from the West).
        let entry = Some(Direction::West);
        let expected = [
            (Direction::East, 8),      // straight on
            (Direction::NorthEast, 3), // 60° left
            (Direction::SouthEast, 3), // 60° right
            (Direction::NorthWest, 1), // 120° left
            (Direction::SouthWest, 1), // 120° right
            (Direction::West, 0),      // U-turn
        ];
        for (dir, weight) in expected {
            assert_eq!(turn_weight(entry, dir, &weights), weight, "{dir:?}");
        }
    }

    #[test]
    fn the_first_step_is_uniform() {
        for dir in Direction::ALL {
            assert_eq!(turn_weight(None, dir, &[8, 3, 1, 0]), 1);
        }
    }

    #[test]
    fn crowded_destinations_weigh_less_than_open_ones() {
        use super::elbow_room;
        use crate::level::coordinates::Coordinates;
        use crate::level::room::Room;

        let from = Coordinates::new(0, 0, 0);
        let open = from.neighbor(Direction::East);

        // An empty level: every neighbour of the target is free except `from`.
        let mut level = Level::new();
        assert_eq!(elbow_room(&level, from, open), 5);

        // Wall the target in completely: no elbow room at all.
        for dir in Direction::ALL {
            let tile = open.neighbor(dir);
            if tile != from {
                level.insert(Room::new(RoomKind::Normal, tile, 0));
            }
        }
        assert_eq!(elbow_room(&level, from, open), 0);
    }
}
