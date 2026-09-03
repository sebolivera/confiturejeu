/// What a room is for; a fixed, compile-time set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RoomKind {
    /// Where the player enters the level.
    Entrance,
    /// An ordinary room.
    Normal,
    /// Holds the level boss.
    Boss,
    /// Holds loot.
    Treasure,
}
