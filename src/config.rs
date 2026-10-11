//! Constantes de configuración y conversión entre coordenadas del juego y del mundo.

use bevy::prelude::*;

/// Separación entre columnas en el plano del mundo (XZ).
pub const SPACING: f32 = 1.2;
/// Altura de cada nivel en el eje Y del mundo.
pub const LEVEL_HEIGHT: f32 = 0.6;
/// Radio de las varillas.
pub const ROD_RADIUS: f32 = 0.045;
/// Grosor de la base de madera.
pub const BASE_THICKNESS: f32 = 0.3;
/// Altura total de las varillas.
pub const ROD_HEIGHT: f32 = 3.0 * LEVEL_HEIGHT + 0.35;
/// Semi-ancho de la base.
pub const BOARD_HALF: f32 = SPACING + 0.55;
/// Altura desde la que cae una pieza.
pub const DROP_START_Y: f32 = ROD_HEIGHT + 1.8;
/// Aceleración de la gravedad usada por la animación de caída (unidades/s²).
pub const FALL_GRAVITY: f32 = 26.0;

/// Punto central del tablero (objetivo de la cámara orbital).
pub const BOARD_CENTER: Vec3 = Vec3::new(0.0, LEVEL_HEIGHT * 1.4, 0.0);

/// Devuelve `(x, z)` de mundo para una columna (índice 0..9).
pub fn column_world_xz(col: usize) -> (f32, f32) {
    let gx = (col / 3) as f32;
    let gy = (col % 3) as f32;
    ((gx - 1.0) * SPACING, (gy - 1.0) * SPACING)
}

/// Devuelve la `y` de mundo para un nivel `z` (0..2).
pub fn level_world_y(level: u8) -> f32 {
    level as f32 * LEVEL_HEIGHT + LEVEL_HEIGHT * 0.5
}

/// Posición de mundo del centro de una celda (columna, nivel).
pub fn cell_world(col: usize, level: u8) -> Vec3 {
    let (x, z) = column_world_xz(col);
    Vec3::new(x, level_world_y(level), z)
}

/// Convierte un punto del plano de la base (y = 0) en una columna (0..9).
pub fn world_to_column(point: Vec3) -> Option<usize> {
    let gx = (point.x / SPACING).round();
    let gy = (point.z / SPACING).round();
    if !(-1.0..=1.0).contains(&gx) || !(-1.0..=1.0).contains(&gy) {
        return None;
    }
    Some(((gx + 1.0) as usize) * 3 + (gy + 1.0) as usize)
}
