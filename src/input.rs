//! Interacción con el ratón: hover de columna y colocación de piezas con clic.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use crate::board::Player;
use crate::config::world_to_column;
use crate::game::{Game, Phase};
use crate::pieces::{spawn_piece, FallingCount, PieceAssets};
use crate::render::BoardVisuals;
use crate::state::AppState;

/// Columna actualmente resaltada bajo el cursor.
#[derive(Resource, Default)]
pub struct HoveredColumn(pub Option<usize>);

/// Distingue un clic de un arrastre de cámara.
#[derive(Resource, Default)]
struct ClickTracker {
    pressed: bool,
    moved: f32,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredColumn>()
            .init_resource::<ClickTracker>()
            .add_systems(
                Update,
                (track_hover, track_click, highlight_hovered)
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Proyecta el cursor sobre el plano de la base (y = 0) para hallar la columna.
fn track_hover(
    window: Single<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut hover: ResMut<HoveredColumn>,
) {
    hover.0 = None;

    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    if let Some(point) =
        ray.plane_intersection_point(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
    {
        hover.0 = world_to_column(point);
    }
}

/// Detecta un clic (poco movimiento) y coloca la pieza del jugador humano.
#[allow(clippy::too_many_arguments)]
fn track_click(
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    mut tracker: ResMut<ClickTracker>,
    hover: Res<HoveredColumn>,
    mut falling: ResMut<FallingCount>,
    assets: Res<PieceAssets>,
    mut game: ResMut<Game>,
    mut commands: Commands,
) {
    if mouse.just_pressed(MouseButton::Left) {
        tracker.pressed = true;
        tracker.moved = 0.0;
    }
    if tracker.pressed && mouse.pressed(MouseButton::Left) {
        tracker.moved += motion.delta.length();
    }
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    let was_click = tracker.pressed && tracker.moved < 8.0;
    tracker.pressed = false;

    if !was_click || game.phase != Phase::HumanTurn || falling.0 > 0 {
        return;
    }
    let Some(col) = hover.0 else {
        return;
    };
    if !game.board.is_legal(col) {
        return;
    }

    let player: Player = game.turn;
    if let Some(level) = game.apply_move(col) {
        spawn_piece(&mut commands, &assets, player, col, level);
        falling.0 += 1;
    }
}

/// Resalta la varilla de la columna bajo el cursor (solo si es jugable).
fn highlight_hovered(
    hover: Res<HoveredColumn>,
    game: Res<Game>,
    visuals: Res<BoardVisuals>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last: Local<Option<usize>>,
) {
    let target = if game.phase == Phase::HumanTurn
        && hover.0.map(|c| game.board.is_legal(c)).unwrap_or(false)
    {
        hover.0
    } else {
        None
    };

    if *last == target {
        return;
    }

    if let Some(previous) = *last {
        if let Some(mut material) = materials.get_mut(&visuals.rod_materials[previous]) {
            material.emissive = LinearRgba::new(0.0, 0.0, 0.0, 1.0);
        }
    }
    if let Some(current) = target {
        if let Some(mut material) = materials.get_mut(&visuals.rod_materials[current]) {
            material.emissive = LinearRgba::new(1.1, 0.75, 0.15, 1.0);
        }
    }
    *last = target;
}
