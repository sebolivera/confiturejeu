//! Top-down level viewer: iterate on the generator without walking levels.
//!
//! Run with `cargo run --bin level_viewer [seed]` (or `just viewer`).
//! Draws the generated level with gizmos only — no meshes, no assets.
//!
//! Controls: arrows pan, `+`/`-` or mouse wheel zoom, `PageUp`/`PageDown`
//! change floor (other floors stay as ghosts), `R` reseeds.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use confiturejeu::level::coordinates::{Coordinates, HEX_RADIUS};
use confiturejeu::level::generation::gen_config::GenConfig;
use confiturejeu::level::generation::generate;
use confiturejeu::level::room::kind::RoomKind;
use confiturejeu::level::svg::edge_corners;
use confiturejeu::level::{Direction, Level};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Fraction of an edge left solid on each side of a door gap.
const DOOR_JAMB: f32 = 0.3;
/// Camera pan speed in world units per second (at zoom 1).
const PAN_SPEED: f32 = 400.0;
/// Keyboard zoom factor per second.
const ZOOM_SPEED: f32 = 1.5;

#[derive(Resource)]
struct Viewer {
    seed: u64,
    level: Level,
    floor: i32,
}

impl Viewer {
    fn from_seed(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let level = generate(&mut rng, &GenConfig::default());
        Self {
            seed,
            level,
            floor: 0,
        }
    }

    /// Lowest and highest floor present in the level.
    fn floor_bounds(&self) -> (i32, i32) {
        self.level
            .rooms
            .keys()
            .fold((0, 0), |(lo, hi), c| (lo.min(c.z), hi.max(c.z)))
    }
}

#[derive(Component)]
struct HudText;

fn main() {
    let seed = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(rand::random::<u64>);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Viewer::from_seed(seed))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, pan_zoom, draw_level, update_hud))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Text::new(""),
        HudText,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Px(15.0),
            ..default()
        },
    ));
}

fn handle_input(mut viewer: ResMut<Viewer>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyR) {
        *viewer = Viewer::from_seed(rand::random());
    }
    let (lo, hi) = viewer.floor_bounds();
    if keys.just_pressed(KeyCode::PageUp) {
        viewer.floor = (viewer.floor + 1).min(hi);
    }
    if keys.just_pressed(KeyCode::PageDown) {
        viewer.floor = (viewer.floor - 1).max(lo);
    }
}

fn pan_zoom(
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    keys: Res<ButtonInput<KeyCode>>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
) {
    let (mut transform, mut projection) = camera.into_inner();
    let Projection::Orthographic(ortho) = &mut *projection else {
        return;
    };

    let mut pan = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowUp) {
        pan.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        pan.y -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        pan.x += 1.0;
    }
    if keys.pressed(KeyCode::ArrowLeft) {
        pan.x -= 1.0;
    }
    let step = PAN_SPEED * ortho.scale * time.delta_secs();
    transform.translation += (pan * step).extend(0.0);

    let mut zoom = -scroll.delta.y * 0.15;
    if keys.pressed(KeyCode::Equal) || keys.pressed(KeyCode::NumpadAdd) {
        zoom = ZOOM_SPEED.mul_add(-time.delta_secs(), zoom);
    }
    if keys.pressed(KeyCode::Minus) || keys.pressed(KeyCode::NumpadSubtract) {
        zoom = ZOOM_SPEED.mul_add(time.delta_secs(), zoom);
    }
    ortho.scale = (ortho.scale * zoom.exp()).clamp(0.05, 20.0);
}

const fn kind_color(kind: RoomKind) -> Color {
    match kind {
        RoomKind::Entrance => Color::srgb(0.26, 0.63, 0.28),
        RoomKind::Normal => Color::srgb(0.69, 0.75, 0.77),
        RoomKind::Boss => Color::srgb(0.90, 0.22, 0.21),
        RoomKind::Treasure => Color::srgb(0.99, 0.85, 0.21),
    }
}

/// Room centre in viewer space: world `x` east, world `-z` mapped to `+y`.
fn center(coords: Coordinates) -> Vec2 {
    let world: Vec3 = coords.into();
    Vec2::new(world.x, -world.z)
}

/// The six corners of a pointy-top hex, matching [`edge_corners`] indexing.
fn corners(c: Vec2) -> [Vec2; 6] {
    core::array::from_fn(|i| {
        let theta = 60.0f32.mul_add(i as f32, 30.0).to_radians();
        c + HEX_RADIUS * Vec2::from_angle(theta)
    })
}

fn draw_level(viewer: Res<Viewer>, mut gizmos: Gizmos) {
    // Ghost floors first so the current floor draws on top.
    let mut rooms: Vec<_> = viewer.level.rooms.values().collect();
    rooms.sort_unstable_by_key(|room| room.coords.z == viewer.floor);

    for room in rooms {
        let color = if room.coords.z == viewer.floor {
            kind_color(room.kind)
        } else {
            kind_color(room.kind).with_alpha(0.15)
        };

        let pts = corners(center(room.coords));
        for dir in Direction::ALL {
            let (i, j) = edge_corners(dir);
            let (a, b) = (pts[i], pts[j]);
            if room.is_open(dir) {
                // Leave a doorway gap in the middle of the shared edge.
                gizmos.line_2d(a, a.lerp(b, DOOR_JAMB), color);
                gizmos.line_2d(a.lerp(b, 1.0 - DOOR_JAMB), b, color);
            } else {
                gizmos.line_2d(a, b, color);
            }
        }
    }
}

fn update_hud(viewer: Res<Viewer>, mut text: Single<&mut Text, With<HudText>>) {
    let (lo, hi) = viewer.floor_bounds();
    ***text = format!(
        "seed  {}\nfloor {} ({lo}..={hi})\n\n\
         [R] reseed  [PgUp/PgDn] floor\n[arrows] pan  [+/-/wheel] zoom",
        viewer.seed, viewer.floor
    );
}
