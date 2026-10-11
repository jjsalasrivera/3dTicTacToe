//! Tres en Raya 3D — un 3x3x3 con gravedad sobre un tablero de madera.
//!
//! `cargo run --release` para jugar.

mod ai;
mod board;
mod camera;
mod config;
mod game;
mod input;
mod pieces;
mod render;
mod state;
mod ui;

use bevy::prelude::*;
use bevy::window::PresentMode;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "3D Tic-Tac-Toe".to_string(),
                resolution: (1100u32, 800u32).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.10)))
        .init_state::<state::AppState>()
        .init_resource::<game::Game>()
        .init_resource::<game::MenuSelection>()
        .add_plugins((
            camera::OrbitCameraPlugin,
            render::BoardPlugin,
            pieces::PiecesPlugin,
            input::InputPlugin,
            ai::AiPlugin,
            ui::UiPlugin,
        ))
        .run();
}
