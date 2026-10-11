//! Estado del tablero 3x3x3, reglas del juego y detección de victoria.
//!
//! Coordenadas: `(x, y, z)` con `x, y, z ∈ {0,1,2}`. El eje Z es el vertical
//! (gravedad): las piezas solo pueden colocarse en el primer nivel libre de su
//! columna `(x, y)`.
//!
//! Cada celda ocupa un bit en un entero de 27 bits: `bit = columna * 3 + nivel`,
//! con `columna = x * 3 + y`. Así, la comprobación de victoria es una operación
//! de bits contra 49 máscaras precomputadas.

use std::collections::HashSet;
use std::sync::LazyLock;

pub const COLUMNS: usize = 9;
pub const MAX_HEIGHT: usize = 3;
pub const CELLS: usize = 27;
/// Columna central de la base `(x = 1, y = 1)`, prohibida en la primera jugada.
pub const CENTER_COLUMN: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Player {
    X,
    O,
}

impl Player {
    pub fn other(self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Player::X => 0,
            Player::O => 1,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Player::X => "X",
            Player::O => "O",
        }
    }
}

/// Índice de bit (0..27) de una celda.
pub fn bit_index(col: usize, level: usize) -> u32 {
    (col * 3 + level) as u32
}

/// Columna (0..9) a partir de `(x, y)`.
pub fn column_of(x: usize, y: usize) -> usize {
    x * 3 + y
}

/// Las 49 líneas ganadoras, como máscaras de 27 bits.
///
/// Se generan probando todas las celdas como origen y las 26 direcciones
/// no nulas de paso `{-1,0,1}³`, quedándose con las ternas dentro del cubo.
pub static LINES: LazyLock<Vec<u32>> = LazyLock::new(build_lines);

fn build_lines() -> Vec<u32> {
    let in_bounds = |x: i32, y: i32, z: i32| {
        (0..3).contains(&x) && (0..3).contains(&y) && (0..3).contains(&z)
    };

    let mut unique: HashSet<u32> = HashSet::new();
    for sx in 0..3i32 {
        for sy in 0..3i32 {
            for sz in 0..3i32 {
                for dx in -1..=1i32 {
                    for dy in -1..=1i32 {
                        for dz in -1..=1i32 {
                            if dx == 0 && dy == 0 && dz == 0 {
                                continue;
                            }
                            let (x1, y1, z1) = (sx + dx, sy + dy, sz + dz);
                            let (x2, y2, z2) = (sx + 2 * dx, sy + 2 * dy, sz + 2 * dz);
                            if !in_bounds(x1, y1, z1) || !in_bounds(x2, y2, z2) {
                                continue;
                            }
                            let a = bit_index(
                                column_of(sx as usize, sy as usize),
                                sz as usize,
                            );
                            let b = bit_index(
                                column_of(x1 as usize, y1 as usize),
                                z1 as usize,
                            );
                            let c = bit_index(
                                column_of(x2 as usize, y2 as usize),
                                z2 as usize,
                            );
                            let mut bits = [a, b, c];
                            bits.sort_unstable();
                            unique.insert((1 << bits[0]) | (1 << bits[1]) | (1 << bits[2]));
                        }
                    }
                }
            }
        }
    }

    let mut lines: Vec<u32> = unique.into_iter().collect();
    lines.sort_unstable();
    lines
}

/// Estado completo del tablero. `cells[columna][nivel]`: 0 = vacío, 1 = X, 2 = O.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Board {
    pub cells: [[u8; MAX_HEIGHT]; COLUMNS],
    pub heights: [u8; COLUMNS],
    pub bits: [u32; 2],
    pub moves: u8,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Self {
        Board {
            cells: [[0; MAX_HEIGHT]; COLUMNS],
            heights: [0; COLUMNS],
            bits: [0; 2],
            moves: 0,
        }
    }

    pub fn height(&self, col: usize) -> usize {
        self.heights[col] as usize
    }

    /// ¿Es legal jugar en esta columna? Incluye la restricción de apertura
    /// (la primera jugada no puede ir al centro de la base).
    pub fn is_legal(&self, col: usize) -> bool {
        if col >= COLUMNS {
            return false;
        }
        if self.height(col) >= MAX_HEIGHT {
            return false;
        }
        if self.moves == 0 && col == CENTER_COLUMN {
            return false;
        }
        true
    }

    /// Coloca una pieza del jugador en la columna; devuelve el nivel ocupado.
    pub fn place(&mut self, col: usize, player: Player) -> Option<u8> {
        if self.height(col) >= MAX_HEIGHT {
            return None;
        }
        let level = self.heights[col];
        self.cells[col][level as usize] = player.index() as u8 + 1;
        self.heights[col] += 1;
        self.moves += 1;
        self.bits[player.index()] |= 1 << bit_index(col, level as usize);
        Some(level)
    }

    /// Retira la pieza superior de una columna; devuelve `(jugador, nivel)`.
    pub fn pop(&mut self, col: usize) -> Option<(Player, u8)> {
        if self.heights[col] == 0 {
            return None;
        }
        let level = self.heights[col] - 1;
        let value = self.cells[col][level as usize];
        let player = if value == 1 { Player::X } else { Player::O };
        self.cells[col][level as usize] = 0;
        self.heights[col] -= 1;
        self.moves -= 1;
        self.bits[player.index()] &= !(1 << bit_index(col, level as usize));
        Some((player, level))
    }

    /// ¿Tiene el jugador tres en línea?
    pub fn is_win(&self, player: Player) -> bool {
        let bits = self.bits[player.index()];
        LINES.iter().any(|mask| (bits & mask) == *mask)
    }

    pub fn is_full(&self) -> bool {
        self.moves as usize >= CELLS
    }

    pub fn cell(&self, col: usize, level: usize) -> Option<Player> {
        match self.cells[col][level] {
            1 => Some(Player::X),
            2 => Some(Player::O),
            _ => None,
        }
    }

    /// Máscara de la línea ganadora del jugador, si existe.
    pub fn winning_line(&self, player: Player) -> Option<u32> {
        let bits = self.bits[player.index()];
        LINES.iter().copied().find(|mask| (bits & mask) == *mask)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hay_49_lineas() {
        assert_eq!(LINES.len(), 49);
    }

    #[test]
    fn diagonal_espacial_cuenta_como_victoria() {
        let mut b = Board::new();
        // (0,0,0) X
        b.place(column_of(0, 0), Player::X);
        // rellenar la columna 0
        b.place(column_of(0, 0), Player::O);
        b.place(column_of(0, 0), Player::O);
        // (1,1,0) O, (1,1,1) X
        b.place(column_of(1, 1), Player::O);
        b.place(column_of(1, 1), Player::X);
        b.place(column_of(1, 1), Player::O);
        // (2,2,0) O, (2,2,1) X? -> necesitamos (2,2,2) X
        b.place(column_of(2, 2), Player::O);
        b.place(column_of(2, 2), Player::X);
        b.place(column_of(2, 2), Player::X);

        assert!(b.is_win(Player::X), "X debe ganar con la diagonal espacial");
    }

    #[test]
    fn la_gravedad_apila_y_limita_la_altura() {
        let mut b = Board::new();
        assert_eq!(b.place(0, Player::X), Some(0));
        assert_eq!(b.place(0, Player::O), Some(1));
        assert_eq!(b.place(0, Player::X), Some(2));
        assert_eq!(b.place(0, Player::O), None, "la columna ya está llena");
    }

    #[test]
    fn el_centro_esta_prohibido_en_la_primera_jugada() {
        let mut b = Board::new();
        assert!(!b.is_legal(CENTER_COLUMN));
        b.place(0, Player::X);
        assert!(b.is_legal(CENTER_COLUMN), "ya permitido en la segunda jugada");
    }

    #[test]
    fn pop_deshace_place() {
        let mut b = Board::new();
        b.place(3, Player::X);
        b.place(3, Player::O);
        assert_eq!(b.pop(3), Some((Player::O, 1)));
        assert!(!b.is_win(Player::O));
        assert_eq!(b.height(3), 1);
        assert_eq!(b.moves, 1);
    }
}
