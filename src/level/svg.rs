//! Pure SVG rendering of a [`Level`], one panel per floor, side by side.
//!
//! No dependencies beyond `std`: dump 50 seeds to files and eyeball them in a
//! browser grid to judge the generator without walking a single level.

use std::fmt::Write;

use crate::level::coordinates::Coordinates;
use crate::level::room::Room;
use crate::level::room::kind::RoomKind;
use crate::level::{Direction, Level};

/// Hex radius in SVG user units; purely cosmetic, unrelated to world scale.
const R: f32 = 24.0;
/// Blank margin around each floor panel.
const PAD: f32 = 24.0;
/// Vertical space reserved for a panel's floor label.
const LABEL_H: f32 = 20.0;
/// Fraction of an edge left solid on each side of a door gap.
const DOOR_JAMB: f32 = 0.3;

/// Fill colour for each room kind.
const fn kind_color(kind: RoomKind) -> &'static str {
    match kind {
        RoomKind::Entrance => "#43a047",
        RoomKind::Normal => "#b0bec5",
        RoomKind::Boss => "#e53935",
        RoomKind::Treasure => "#fdd835",
    }
}

/// Corner indices (into [`corners`]) bounding the edge shared with each
/// [`Direction`] neighbour of a pointy-top hex.
#[must_use]
pub const fn edge_corners(dir: Direction) -> (usize, usize) {
    match dir {
        Direction::NorthEast => (0, 1),
        Direction::NorthWest => (1, 2),
        Direction::West => (2, 3),
        Direction::SouthWest => (3, 4),
        Direction::SouthEast => (4, 5),
        Direction::East => (5, 0),
    }
}

/// Room centre in SVG space (y grows downwards, i.e. towards world south).
fn center(coords: Coordinates) -> (f32, f32) {
    let x = R * 3.0_f32.sqrt() * (coords.r as f32).mul_add(0.5, coords.q as f32);
    let y = R * 1.5 * coords.r as f32;
    (x, y)
}

/// The six corners of a pointy-top hex, in SVG (y-down) space.
///
/// Corner `i` sits at `30° + 60°·i` counter-clockwise from east, north up.
fn corners(cx: f32, cy: f32) -> [(f32, f32); 6] {
    core::array::from_fn(|i| {
        let theta = 60.0f32.mul_add(i as f32, 30.0).to_radians();
        (R.mul_add(theta.cos(), cx), R.mul_add(-theta.sin(), cy))
    })
}

fn line(out: &mut String, (ax, ay): (f32, f32), (bx, by): (f32, f32)) {
    let _ = writeln!(
        out,
        "<line x1=\"{ax:.1}\" y1=\"{ay:.1}\" x2=\"{bx:.1}\" y2=\"{by:.1}\"/>"
    );
}

/// Emits one room: filled hex, edges (with a door gap where open), stair marks.
fn draw_room(out: &mut String, level: &Level, room: &Room, ox: f32, oy: f32) {
    let (cx, cy) = center(room.coords);
    let (cx, cy) = (cx + ox, cy + oy);
    let pts = corners(cx, cy);

    let _ = write!(out, "<polygon points=\"");
    for (x, y) in pts {
        let _ = write!(out, "{x:.1},{y:.1} ");
    }
    let _ = writeln!(
        out,
        "\" fill=\"{}\" fill-opacity=\"0.5\" stroke=\"none\"/>",
        kind_color(room.kind)
    );

    for dir in Direction::ALL {
        let (i, j) = edge_corners(dir);
        let ((ax, ay), (bx, by)) = (pts[i], pts[j]);
        if room.is_open(dir) {
            // Leave a doorway gap in the middle of the shared edge.
            let lerp = |t: f32| ((bx - ax).mul_add(t, ax), (by - ay).mul_add(t, ay));
            line(out, (ax, ay), lerp(DOOR_JAMB));
            line(out, lerp(1.0 - DOOR_JAMB), (bx, by));
        } else {
            line(out, (ax, ay), (bx, by));
        }
    }

    let Coordinates { q, r, z } = room.coords;
    let mut marker = String::new();
    if level.rooms.contains_key(&Coordinates::new(q, r, z + 1)) {
        marker.push('▲');
    }
    if level.rooms.contains_key(&Coordinates::new(q, r, z - 1)) {
        marker.push('▼');
    }
    if !marker.is_empty() {
        let _ = writeln!(
            out,
            "<text x=\"{cx:.1}\" y=\"{cy:.1}\" text-anchor=\"middle\" \
             dominant-baseline=\"central\" font-size=\"{:.0}\">{marker}</text>",
            R * 0.6
        );
    }
}

/// Renders the level as a standalone SVG document, one panel per floor.
///
/// Hexes are outlined and filled per [`RoomKind`], doors show as gaps on the
/// shared edge, and rooms with a room directly above/below get ▲/▼ markers.
/// Output is deterministic for a given level, so dumps are diffable.
#[must_use]
pub fn level_to_svg(level: &Level) -> String {
    let mut floors: Vec<i32> = level.rooms.keys().map(|c| c.z).collect();
    floors.sort_unstable();
    floors.dedup();

    let mut body = String::new();
    let mut panel_x = PAD;
    let mut height = 2.0 * PAD;

    for &z in &floors {
        let mut rooms: Vec<&Room> = level.rooms.values().filter(|r| r.coords.z == z).collect();
        rooms.sort_unstable_by_key(|r| (r.coords.r, r.coords.q));

        let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
        let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);
        for room in &rooms {
            let (x, y) = center(room.coords);
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }

        let _ = writeln!(
            body,
            "<text x=\"{panel_x:.1}\" y=\"{:.1}\" font-size=\"14\">floor z = {z}</text>",
            PAD + 12.0
        );

        // Shift this floor's content so its bounding box starts at the panel.
        let ox = panel_x - (min_x - R);
        let oy = PAD + LABEL_H - (min_y - R);
        for room in rooms {
            draw_room(&mut body, level, room, ox, oy);
        }

        panel_x += 2.0f32.mul_add(R, max_x - min_x) + PAD;
        height = height.max(2.0f32.mul_add(PAD, 2.0f32.mul_add(R, max_y - min_y) + LABEL_H));
    }

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{panel_x:.0}\" height=\"{height:.0}\" \
         font-family=\"monospace\">\n<g stroke=\"#333\" stroke-width=\"2\" \
         stroke-linecap=\"round\">\n{body}</g>\n</svg>\n"
    )
}

#[cfg(test)]
mod tests {
    use super::level_to_svg;
    use crate::level::coordinates::Coordinates;
    use crate::level::room::Room;
    use crate::level::room::kind::RoomKind;
    use crate::level::{Direction, Level};

    fn count(haystack: &str, needle: &str) -> usize {
        haystack.matches(needle).count()
    }

    #[test]
    fn an_empty_level_is_a_valid_empty_document() {
        let svg = level_to_svg(&Level::new());
        assert!(svg.starts_with("<svg "));
        assert!(svg.trim_end().ends_with("</svg>"));
        assert_eq!(count(&svg, "<polygon"), 0);
        assert_eq!(count(&svg, "<line"), 0);
    }

    #[test]
    fn a_sealed_room_is_a_hex_with_six_solid_edges() {
        let mut level = Level::new();
        level.insert(Room::new(RoomKind::Boss, Coordinates::new(0, 0, 0), 0));

        let svg = level_to_svg(&level);
        assert_eq!(count(&svg, "<polygon"), 1);
        assert_eq!(count(&svg, "<line"), 6);
        assert!(svg.contains("#e53935"), "boss colour missing:\n{svg}");
    }

    #[test]
    fn each_door_splits_its_edge_into_two_segments() {
        let mut level = Level::new();
        let openings = Direction::East.mask() | Direction::NorthWest.mask();
        level.insert(Room::new(
            RoomKind::Normal,
            Coordinates::new(0, 0, 0),
            openings,
        ));

        // 4 solid edges + 2 doors drawn as 2 segments each.
        assert_eq!(count(&level_to_svg(&level), "<line"), 8);
    }

    #[test]
    fn floors_get_one_labelled_panel_each_and_stair_markers() {
        let mut level = Level::new();
        level.insert(Room::new(RoomKind::Entrance, Coordinates::new(0, 0, 0), 0));
        level.insert(Room::new(RoomKind::Normal, Coordinates::new(0, 0, 1), 0));

        let svg = level_to_svg(&level);
        assert!(svg.contains("floor z = 0"));
        assert!(svg.contains("floor z = 1"));
        assert_eq!(count(&svg, "▲"), 1);
        assert_eq!(count(&svg, "▼"), 1);
    }

    #[test]
    fn output_is_independent_of_insertion_order() {
        let rooms = [
            Room::new(RoomKind::Entrance, Coordinates::new(0, 0, 0), 0b10),
            Room::new(RoomKind::Normal, Coordinates::new(1, 0, 0), 0b10_0010),
            Room::new(RoomKind::Boss, Coordinates::new(2, 0, 0), 0b1_0000),
        ];

        let mut forward = Level::new();
        let mut backward = Level::new();
        for room in rooms {
            forward.insert(room);
        }
        for room in rooms.into_iter().rev() {
            backward.insert(room);
        }

        assert_eq!(level_to_svg(&forward), level_to_svg(&backward));
    }
}
