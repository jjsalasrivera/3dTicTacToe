//! Piezas del juego: creación de meshes/materiales, colocación y animación de
//! caída.

use bevy::prelude::*;

use crate::board::Player;
use crate::config::*;

/// Meshes y materiales compartidos por todas las piezas.
#[derive(Resource)]
pub struct PieceAssets {
    pub x_bar_mesh: Handle<Mesh>,
    pub o_mesh: Handle<Mesh>,
    pub x_material: Handle<StandardMaterial>,
    pub o_material: Handle<StandardMaterial>,
}

/// Marca una pieza colocada en el tablero.
#[derive(Component)]
#[allow(dead_code)]
pub struct Piece {
    pub col: usize,
    pub player: Player,
}

/// Estado de caída de una pieza recién colocada.
#[derive(Component)]
pub struct Falling {
    pub velocity: f32,
    pub target_y: f32,
    pub bounces: u8,
}

/// Número de piezas actualmente cayendo (bloquea la interacción).
#[derive(Resource, Default)]
pub struct FallingCount(pub u32);

pub struct PiecesPlugin;

impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FallingCount>()
            .add_systems(Startup, setup_piece_assets)
            .add_systems(Update, update_falling);
    }
}

fn setup_piece_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(PieceAssets {
        // Una barra de la "X"; se colocan dos cruzadas.
        x_bar_mesh: meshes.add(Cuboid::new(0.58, 0.16, 0.16)),
        // Anillo ("O") tumbado, con el eje en Y para que la varilla lo atraviese.
        o_mesh: meshes.add(Torus::new(0.16, 0.40)),
        x_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.82, 0.22, 0.18),
            metallic: 0.3,
            perceptual_roughness: 0.4,
            ..default()
        }),
        o_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.48, 0.90),
            metallic: 0.3,
            perceptual_roughness: 0.4,
            ..default()
        }),
    });
}

/// Crea la entidad de una pieza que cae hasta `level` en la columna `col`.
pub fn spawn_piece(
    commands: &mut Commands,
    assets: &PieceAssets,
    player: Player,
    col: usize,
    level: u8,
) {
    let target = cell_world(col, level);
    let start = Vec3::new(target.x, DROP_START_Y, target.z);

    let mut entity = commands.spawn((
        Transform::from_translation(start),
        Visibility::default(),
        Piece { col, player },
        Falling {
            velocity: 0.0,
            target_y: target.y,
            bounces: 2,
        },
    ));

    match player {
        Player::X => {
            entity.with_children(|parent| {
                for angle in [std::f32::consts::FRAC_PI_4, -std::f32::consts::FRAC_PI_4] {
                    parent.spawn((
                        Mesh3d(assets.x_bar_mesh.clone()),
                        MeshMaterial3d(assets.x_material.clone()),
                        Transform::from_rotation(Quat::from_rotation_y(angle)),
                    ));
                }
            });
        }
        Player::O => {
            entity.with_children(|parent| {
                parent.spawn((
                    Mesh3d(assets.o_mesh.clone()),
                    MeshMaterial3d(assets.o_material.clone()),
                ));
            });
        }
    }
}

fn update_falling(
    time: Res<Time>,
    mut commands: Commands,
    mut falling_count: ResMut<FallingCount>,
    mut query: Query<(Entity, &mut Transform, &mut Falling)>,
) {
    let dt = time.delta_secs().min(0.05);

    for (entity, mut transform, mut falling) in &mut query {
        falling.velocity -= FALL_GRAVITY * dt;
        let next_y = transform.translation.y + falling.velocity * dt;

        if next_y <= falling.target_y {
            transform.translation.y = falling.target_y;
            if falling.bounces > 0 && falling.velocity < -1.0 {
                falling.bounces -= 1;
                falling.velocity = -falling.velocity * 0.32;
            } else {
                commands.entity(entity).remove::<Falling>();
                falling_count.0 = falling_count.0.saturating_sub(1);
            }
        } else {
            transform.translation.y = next_y;
        }
    }
}
