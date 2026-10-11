//! Cámara orbital alrededor del tablero (arrastrar con el ratón + rueda).

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::config::BOARD_CENTER;

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        PanOrbitCamera {
            focus: BOARD_CENTER,
            target_focus: BOARD_CENTER,
            radius: Some(9.5),
            yaw: Some(0.85),
            pitch: Some(0.7),
            zoom_lower_limit: 4.0,
            zoom_upper_limit: Some(18.0),
            allow_upside_down: false,
            ..default()
        },
        Transform::from_xyz(6.0, 7.0, 7.0).looking_at(BOARD_CENTER, Vec3::Y),
    ));
}
