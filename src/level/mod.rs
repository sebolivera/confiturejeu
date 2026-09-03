/// Hexagonal coordinates module
pub mod coordinates;
pub mod generation;
pub mod level_config;
/// Hexagonal room module
pub mod room;

use crate::level::coordinates::Coordinates;
use crate::level::room::Room;
use rand::{Rng, RngExt};
use std::collections::HashMap;

/// Level of hexagonal rooms
#[derive(Debug)]
pub struct Level {
    /// Rooms in a level
    pub rooms: HashMap<Coordinates, Room>,
}

impl Level {
    /// Default constructor
    #[must_use]
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    /// Adds a room, keyed by its own coordinates so the map key and
    /// [`Room::coords`] cannot desync. Returns the room previously there, if any.
    pub fn insert(&mut self, room: Room) -> Option<Room> {
        self.rooms.insert(room.coords, room)
    }
}

impl Default for Level {
    fn default() -> Self {
        Self::new()
    }
}

/// Hexagonal "directions" for rooms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Towards `+x`, `-z`.
    NorthEast = 0,
    /// Towards `+x`.
    East = 1,
    /// Towards `+x`, `+z`.
    SouthEast = 2,
    /// Towards `-x`, `+z`.
    SouthWest = 3,
    /// Towards `-x`.
    West = 4,
    /// Towards `-x`, `-z`.
    NorthWest = 5,
}

impl Direction {
    /// Every direction, in discriminant order.
    ///
    /// This is the canonical iteration order for anything indexed by direction.
    pub const ALL: [Self; 6] = [
        Self::NorthEast,
        Self::East,
        Self::SouthEast,
        Self::SouthWest,
        Self::West,
        Self::NorthWest,
    ];

    /// Axial `(q, r)` step taken when moving one tile in this direction.
    ///
    /// Single source of truth for the direction/coordinate mapping.
    #[must_use]
    pub const fn offset(self) -> (i32, i32) {
        match self {
            Self::NorthEast => (1, -1),
            Self::East => (1, 0),
            Self::SouthEast => (0, 1),
            Self::SouthWest => (-1, 1),
            Self::West => (-1, 0),
            Self::NorthWest => (0, -1),
        }
    }

    /// The direction facing back the other way.
    ///
    /// Needed to keep openings reciprocal: our east opening must be our neighbour's west one.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::NorthEast => Self::SouthWest,
            Self::East => Self::West,
            Self::SouthEast => Self::NorthWest,
            Self::SouthWest => Self::NorthEast,
            Self::West => Self::East,
            Self::NorthWest => Self::SouthEast,
        }
    }

    /// Bit mask of this direction within an `openings` bitfield.
    #[must_use]
    pub const fn mask(self) -> u8 {
        1 << (self as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, Level};
    use crate::level::coordinates::Coordinates;
    use crate::level::room::Room;
    use crate::level::room::kind::RoomKind;
    use bevy::prelude::Vec3;

    #[test]
    fn a_new_level_has_no_rooms() {
        let level = Level::new();
        assert!(level.rooms.is_empty());
    }

    #[test]
    fn rooms_are_retrievable_by_their_coordinates() {
        let mut level = Level::new();
        let coords = Coordinates::new(1, -2, 0);
        let room = Room {
            kind: RoomKind::Normal,
            coords,
            openings: 0,
        };

        level.insert(room);

        assert_eq!(level.rooms.len(), 1);
        assert_eq!(level.rooms.get(&coords).map(|r| r.coords), Some(coords));
        assert!(!level.rooms.contains_key(&Coordinates::new(0, 0, 0)));
    }

    #[test]
    fn all_is_in_discriminant_order() {
        for (index, dir) in Direction::ALL.into_iter().enumerate() {
            assert_eq!(dir as usize, index, "{dir:?} is out of order in ALL");
        }
    }

    #[test]
    fn neighbors_are_indexed_by_direction() {
        let origin = Coordinates::new(3, -2, 0);
        let neighbors = origin.neighbors();
        for dir in Direction::ALL {
            assert_eq!(neighbors[dir as usize], origin.neighbor(dir));
        }
    }

    #[test]
    fn opposite_round_trips() {
        let origin = Coordinates::new(3, -2, 0);
        for dir in Direction::ALL {
            assert_eq!(dir.opposite().opposite(), dir);
            assert_ne!(dir.opposite(), dir);
            assert_eq!(origin.neighbor(dir).neighbor(dir.opposite()), origin);
        }
    }

    #[test]
    fn masks_are_distinct_bits() {
        let mut seen = 0_u8;
        for dir in Direction::ALL {
            assert_eq!(seen & dir.mask(), 0, "{dir:?} reuses a bit");
            seen |= dir.mask();
        }
        assert_eq!(seen, 0b0011_1111);
    }

    /// Compass names must match the pointy-top world layout: `+x` east, `-z` north.
    #[test]
    fn offsets_match_world_positions() {
        fn sign(value: f32) -> i32 {
            if value > f32::EPSILON {
                1
            } else if value < -f32::EPSILON {
                -1
            } else {
                0
            }
        }

        let origin = Coordinates::new(0, 0, 0);
        let expected = [
            (Direction::NorthEast, (1, 1)),
            (Direction::East, (1, 0)),
            (Direction::SouthEast, (1, -1)),
            (Direction::SouthWest, (-1, -1)),
            (Direction::West, (-1, 0)),
            (Direction::NorthWest, (-1, 1)),
        ];

        for (dir, (east, north)) in expected {
            let world: Vec3 = origin.neighbor(dir).into();
            assert_eq!(
                sign(world.x),
                east,
                "{dir:?} has the wrong east/west sign (x = {})",
                world.x
            );
            assert_eq!(
                sign(-world.z),
                north,
                "{dir:?} has the wrong north/south sign (z = {})",
                world.z
            );
        }
    }
}
