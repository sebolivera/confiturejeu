use crate::level::Direction;
use bevy::prelude::Vec3;

/// Distance from the centre of a hex to one of its corners, in world units.
pub const HEX_RADIUS: f32 = 17.0;

/// Vertical distance between two floors, in world units.
pub const FLOOR_HEIGHT: f32 = 30.0;

/// Axial coordinates of a hexagonal tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Coordinates {
    /// Column axis.
    pub q: i32,
    /// Row axis.
    pub r: i32,
    /// Vertical position.
    pub z: i32,
}

impl Coordinates {
    /// Build coordinates from an axial pair.
    #[must_use]
    pub const fn new(q: i32, r: i32, z: i32) -> Self {
        Self { q, r, z }
    }

    /// The adjacent tile in a given direction.
    #[must_use]
    pub const fn neighbor(self, dir: Direction) -> Self {
        let (dq, dr) = dir.offset();
        Self {
            q: self.q + dq,
            r: self.r + dr,
            z: self.z,
        }
    }

    /// All six adjacent tiles, indexed by [`Direction`] discriminant.
    #[must_use]
    pub fn neighbors(self) -> [Self; 6] {
        Direction::ALL.map(|dir| self.neighbor(dir))
    }
}

/// From hex coordinates to vec3
impl From<Coordinates> for Vec3 {
    fn from(hex: Coordinates) -> Self {
        let x = HEX_RADIUS * 3.0_f32.sqrt() * (hex.q as f32 + hex.r as f32 / 2.0);
        let y = FLOOR_HEIGHT * hex.z as f32;
        let z = HEX_RADIUS * (3.0 / 2.0) * hex.r as f32;

        Self::new(x, y, z)
    }
}
