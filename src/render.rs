//! Renderizado del tablero: base de madera, nueve varillas, luces y la línea
//! ganadora.

use bevy::prelude::*;

use crate::board::COLUMNS;
use crate::config::*;
use crate::game::{Game, Outcome};
use crate::state::AppState;

/// Marca una columna (varilla) con su índice.
#[derive(Component)]
pub struct Rod;

/// Materiales de las varillas, para poder resaltarlas al pasar el ratón.
#[derive(Resource)]
pub struct BoardVisuals {
    pub rod_materials: Vec<Handle<StandardMaterial>>,
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalAmbientLight {
            color: Color::srgb(1.0, 0.96, 0.9),
            brightness: 350.0,
            ..default()
        })
        .add_systems(Startup, setup_board)
        .add_systems(
            Update,
            draw_win_line.run_if(in_state(AppState::Playing).or_else(in_state(AppState::GameOver))),
        );
    }
}

fn setup_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Base de madera.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            BOARD_HALF * 2.0,
            BASE_THICKNESS,
            BOARD_HALF * 2.0,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.28, 0.15),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -BASE_THICKNESS * 0.5, 0.0),
    ));

    // Rebaje interior (da profundidad visual).
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            BOARD_HALF * 2.0 - 0.30,
            0.06,
            BOARD_HALF * 2.0 - 0.30,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.30, 0.17, 0.08),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.03, 0.0),
    ));

    let rod_mesh = meshes.add(Cylinder::new(ROD_RADIUS, ROD_HEIGHT));
    let cap_mesh = meshes.add(Sphere::new(ROD_RADIUS * 1.7));
    let socket_mesh = meshes.add(Cylinder::new(0.30, 0.06));
    let socket_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.12, 0.06),
        perceptual_roughness: 1.0,
        ..default()
    });

    let mut rod_materials = Vec::with_capacity(COLUMNS);
    for col in 0..COLUMNS {
        let (x, z) = column_world_xz(col);
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.34, 0.21, 0.11),
            perceptual_roughness: 0.6,
            metallic: 0.2,
            ..default()
        });
        rod_materials.push(material.clone());

        commands.spawn((
            Mesh3d(rod_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(x, ROD_HEIGHT * 0.5, z),
            Rod,
        ));
        // Remate superior de la varilla.
        commands.spawn((
            Mesh3d(cap_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(x, ROD_HEIGHT, z),
        ));
        // Aro de la base (marca visual de la columna).
        commands.spawn((
            Mesh3d(socket_mesh.clone()),
            MeshMaterial3d(socket_mat.clone()),
            Transform::from_xyz(x, 0.05, z),
        ));
    }
    commands.insert_resource(BoardVisuals { rod_materials });

    // Luces.
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 12.0, 6.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 120_000.0,
            color: Color::srgb(1.0, 0.95, 0.85),
            ..default()
        },
        Transform::from_xyz(-4.0, 6.0, -3.0),
    ));
}

/// Dibuja la línea ganadora cuando la partida ha terminado con victoria.
fn draw_win_line(game: Res<Game>, mut gizmos: Gizmos) {
    let Some(Outcome::Win(player)) = game.outcome else {
        return;
    };
    let Some(mask) = game.board.winning_line(player) else {
        return;
    };

    let mut points = Vec::new();
    for bit in 0..27u32 {
        if mask & (1 << bit) != 0 {
            let col = (bit / 3) as usize;
            let level = (bit % 3) as u8;
            points.push(cell_world(col, level));
        }
    }
    if points.len() == 3 {
        let color = Color::srgb(1.0, 0.85, 0.2);
        gizmos.line(points[0], points[2], color);
        for p in points {
            gizmos.sphere(p, 0.13, color);
        }
    }
}
