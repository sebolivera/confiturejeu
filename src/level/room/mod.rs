/// Room kinds.
pub mod kind;

use crate::level::Direction;
use crate::level::coordinates::Coordinates;
use crate::level::room::kind::RoomKind;

/// Hexagonal room
#[derive(Clone, Copy, Debug)]
pub struct Room {
    /// What the room is for.
    pub kind: RoomKind,
    /// Where the room sits on the grid.
    pub coords: Coordinates,
    /// Bitmask of open sides, one bit per [`Direction`].
    pub openings: u8,
}

impl Room {
    /// Check which adjacent rooms are accessible
    #[must_use]
    pub fn accessible_neighbors(&self) -> Vec<Coordinates> {
        self.open_directions()
            .map(|dir| self.coords.neighbor(dir))
            .collect()
    }

    /// Helper to easily check a specific side
    #[must_use]
    pub const fn is_open(&self, dir: Direction) -> bool {
        (self.openings & dir.mask()) != 0
    }

    /// Every direction this room is open towards.
    pub fn open_directions(&self) -> impl Iterator<Item = Direction> + '_ {
        Direction::ALL.into_iter().filter(|dir| self.is_open(*dir))
    }
}

#[cfg(test)]
mod tests {
    use super::{Room, RoomKind};
    use crate::level::Direction;
    use crate::level::coordinates::Coordinates;

    fn room_open_towards(coords: Coordinates, dirs: &[Direction]) -> Room {
        let openings = dirs.iter().fold(0, |acc, dir| acc | dir.mask());
        Room {
            kind: RoomKind::Normal,
            coords,
            openings,
        }
    }

    #[test]
    fn accessible_neighbors_follow_the_open_sides() {
        let coords = Coordinates::new(2, -1, 0);
        let open = [Direction::East, Direction::NorthWest];
        let room = room_open_towards(coords, &open);

        let expected: Vec<_> = open.iter().map(|dir| coords.neighbor(*dir)).collect();
        assert_eq!(room.accessible_neighbors(), expected);
    }

    #[test]
    fn a_sealed_room_leads_nowhere() {
        let room = room_open_towards(Coordinates::new(0, 0, 0), &[]);
        assert!(room.accessible_neighbors().is_empty());
    }

    #[test]
    fn a_fully_open_room_reaches_every_neighbor() {
        let coords = Coordinates::new(-3, 4, 0);
        let room = room_open_towards(coords, &Direction::ALL);
        assert_eq!(room.accessible_neighbors(), coords.neighbors().to_vec());
    }

    #[test]
    fn is_open_reports_exactly_the_open_sides() {
        let open = [Direction::East, Direction::SouthWest];
        let room = room_open_towards(Coordinates::new(0, 0, 0), &open);

        for dir in Direction::ALL {
            assert_eq!(room.is_open(dir), open.contains(&dir), "{dir:?}");
        }
    }

    #[test]
    fn a_sealed_room_has_no_open_side() {
        let room = room_open_towards(Coordinates::new(0, 0, 0), &[]);
        for dir in Direction::ALL {
            assert!(!room.is_open(dir), "{dir:?}");
        }
    }

    #[test]
    fn open_directions_yield_the_open_sides_in_order() {
        // Deliberately out of ALL order: iteration follows ALL, not insertion.
        let open = [Direction::West, Direction::East];
        let room = room_open_towards(Coordinates::new(0, 0, 0), &open);

        let dirs: Vec<_> = room.open_directions().collect();
        let expected: Vec<_> = Direction::ALL
            .into_iter()
            .filter(|dir| open.contains(dir))
            .collect();
        assert_eq!(dirs, expected);
    }

    #[test]
    fn open_directions_of_a_fully_open_room_cover_all() {
        let room = room_open_towards(Coordinates::new(0, 0, 0), &Direction::ALL);
        let dirs: Vec<_> = room.open_directions().collect();
        assert_eq!(dirs, Direction::ALL.to_vec());
    }

    /// Openings must be reciprocal: if we open east, the room east of us must open west.
    #[test]
    fn openings_pair_up_with_the_neighbor() {
        let coords = Coordinates::new(1, 1, 0);
        for dir in Direction::ALL {
            let room = room_open_towards(coords, &[dir]);
            let neighbor = room_open_towards(coords.neighbor(dir), &[dir.opposite()]);

            assert_eq!(room.accessible_neighbors(), vec![neighbor.coords]);
            assert_eq!(neighbor.accessible_neighbors(), vec![room.coords]);
        }
    }
}
