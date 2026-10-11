//! Inteligencia artificial: negamax con poda alfa-beta, tabla de transposición
//! (hashing Zobrist) y profundización iterativa.
//!
//! El factor de ramificación es ≤ 9 (una jugada por columna no llena), lo que
//! hace que la búsqueda sea muy asequible incluso con profundidades altas.

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

use bevy::prelude::*;

use crate::board::{Board, Player, CENTER_COLUMN, COLUMNS, LINES, MAX_HEIGHT};
use crate::game::{Difficulty, Game, Phase};
use crate::pieces::{spawn_piece, FallingCount, PieceAssets};
use crate::state::AppState;

const WIN_SCORE: i32 = 1_000_000;

/// Tabla Zobrist: para cada celda (27) y jugador (2), un valor aleatorio fijo.
static ZOBRIST: LazyLock<[[u64; 2]; 27]> = LazyLock::new(|| {
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut table = [[0u64; 2]; 27];
    for entry in table.iter_mut() {
        entry[0] = next();
        entry[1] = next();
    }
    table
});

/// Posición ligera para la búsqueda: tablero + turno + hash incremental.
struct Pos {
    board: Board,
    turn: Player,
    hash: u64,
}

impl Pos {
    fn new(board: &Board, turn: Player) -> Self {
        let mut hash = 0;
        for col in 0..COLUMNS {
            for level in 0..MAX_HEIGHT {
                if let Some(player) = board.cell(col, level) {
                    hash ^= ZOBRIST[col * 3 + level][player.index()];
                }
            }
        }
        Pos {
            board: *board,
            turn,
            hash,
        }
    }

    fn legal_moves(&self) -> Vec<usize> {
        let mut moves = Vec::with_capacity(COLUMNS);
        for col in 0..COLUMNS {
            if self.board.height(col) < MAX_HEIGHT
                && !(self.board.moves == 0 && col == CENTER_COLUMN)
            {
                moves.push(col);
            }
        }
        moves
    }

    fn apply(&mut self, col: usize) -> u8 {
        let level = self.board.heights[col];
        let player = self.turn;
        self.board.place(col, player);
        self.hash ^= ZOBRIST[col * 3 + level as usize][player.index()];
        self.turn = player.other();
        level
    }

    fn undo(&mut self, col: usize) {
        let (player, level) = self.board.pop(col).expect("undo de jugada inexistente");
        self.turn = self.turn.other();
        self.hash ^= ZOBRIST[col * 3 + level as usize][player.index()];
    }
}

#[derive(Clone, Copy)]
struct TTEntry {
    depth: i32,
    score: i32,
    /// 0 = exacto, 1 = cota inferior, 2 = cota superior.
    flag: u8,
}

struct Searcher {
    tt: HashMap<u64, TTEntry>,
    nodes: u64,
    max_nodes: u64,
    aborted: bool,
    root_depth: i32,
}

impl Searcher {
    fn new(max_nodes: u64) -> Self {
        Searcher {
            tt: HashMap::new(),
            nodes: 0,
            max_nodes,
            aborted: false,
            root_depth: 0,
        }
    }

    fn negamax(&mut self, pos: &mut Pos, depth: i32, mut alpha: i32, mut beta: i32) -> i32 {
        self.nodes += 1;
        if self.nodes > self.max_nodes {
            self.aborted = true;
            return 0;
        }

        let alpha_orig = alpha;
        let key = pos.hash;
        if let Some(entry) = self.tt.get(&key) {
            if entry.depth >= depth {
                match entry.flag {
                    0 => return entry.score,
                    1 => {
                        if entry.score > alpha {
                            alpha = entry.score;
                        }
                    }
                    _ => {
                        if entry.score < beta {
                            beta = entry.score;
                        }
                    }
                }
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        if depth <= 0 {
            return evaluate(&pos.board, pos.turn);
        }

        let mut moves = pos.legal_moves();
        if moves.is_empty() {
            return 0; // tablero lleno -> empate
        }
        moves.sort_by_key(|&col| priority(col));

        let mut best = i32::MIN + 1;
        for col in moves {
            let me = pos.turn;
            pos.apply(col);
            let score = if pos.board.is_win(me) {
                WIN_SCORE - (self.root_depth - depth)
            } else {
                -self.negamax(pos, depth - 1, -beta, -alpha)
            };
            pos.undo(col);

            if self.aborted {
                return best.max(0);
            }
            if score > best {
                best = score;
            }
            if best > alpha {
                alpha = best;
            }
            if alpha >= beta {
                break;
            }
        }

        let flag = if best <= alpha_orig {
            2
        } else if best >= beta {
            1
        } else {
            0
        };
        self.tt.insert(key, TTEntry { depth, score: best, flag });
        best
    }
}

/// Orden heurístico de movimientos: centro, esquinas, bordes.
fn priority(col: usize) -> i32 {
    if col == CENTER_COLUMN {
        0
    } else if matches!(col, 0 | 2 | 6 | 8) {
        1
    } else {
        2
    }
}

/// Evaluación heurística desde la perspectiva del jugador al que le toca mover.
///
/// Para cada línea: si contiene piezas de ambos jugadores está muerta; si solo
/// contiene piezas de uno, suma (2 en línea vale mucho, 1 vale poco).
fn evaluate(board: &Board, player: Player) -> i32 {
    let me = board.bits[player.index()];
    let opponent = board.bits[player.other().index()];
    let mut score = 0;

    for &mask in LINES.iter() {
        let mine = (me & mask).count_ones() as i32;
        let theirs = (opponent & mask).count_ones() as i32;
        if mine != 0 && theirs != 0 {
            continue;
        }
        if mine != 0 {
            score += if mine >= 2 { 80 } else { 4 };
        } else if theirs != 0 {
            score -= if theirs >= 2 { 80 } else { 4 };
        }
    }
    score
}

/// PRNG xorshift sencillo, sin dependencias externas.
fn rand_u32() -> u32 {
    static STATE: AtomicU64 = AtomicU64::new(0);
    let mut s = STATE.load(Ordering::Relaxed);
    if s == 0 {
        use std::time::{SystemTime, UNIX_EPOCH};
        s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x2545_F491_4F6C_DD1D)
            | 1;
    }
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    STATE.store(s, Ordering::Relaxed);
    (s >> 32) as u32
}

/// Calcula la mejor jugada para `player` sobre `board`.
pub fn best_move(board: &Board, player: Player, difficulty: Difficulty) -> Option<usize> {
    let mut pos = Pos::new(board, player);
    let moves = pos.legal_moves();
    if moves.is_empty() {
        return None;
    }
    if moves.len() == 1 {
        return Some(moves[0]);
    }

    // El nivel fácil juega mal a propósito de vez en cuando.
    if difficulty == Difficulty::Easy && rand_u32() % 100 < 30 {
        return Some(moves[(rand_u32() as usize) % moves.len()]);
    }

    let (max_depth, max_nodes) = match difficulty {
        Difficulty::Easy => (3, 40_000),
        Difficulty::Medium => (6, 250_000),
        Difficulty::Hard => (12, 800_000),
    };

    let mut ordered = moves.clone();
    ordered.sort_by_key(|&col| priority(col));

    let mut searcher = Searcher::new(max_nodes);
    let mut best = ordered[0];

    for depth in 1..=max_depth {
        searcher.root_depth = depth;
        let mut local_best = ordered[0];
        let mut local_score = i32::MIN + 1;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX - 1;

        for &col in &ordered {
            let me = pos.turn;
            pos.apply(col);
            let score = if pos.board.is_win(me) {
                WIN_SCORE
            } else {
                -searcher.negamax(&mut pos, depth - 1, -beta, -alpha)
            };
            pos.undo(col);

            if searcher.aborted {
                break;
            }
            if score > local_score {
                local_score = score;
                local_best = col;
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                break;
            }
        }

        if searcher.aborted {
            break;
        }

        best = local_best;

        // Reordena para la siguiente iteración: la mejor jugada primero.
        ordered.retain(|&col| col != best);
        ordered.insert(0, best);

        if local_score >= WIN_SCORE - 10 {
            break; // victoria encontrada: no hace falta seguir
        }
    }

    Some(best)
}

// ---------------------------------------------------------------------------
// Integración con Bevy
// ---------------------------------------------------------------------------

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ai_turn.run_if(in_state(AppState::Playing)));
    }
}

fn ai_turn(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut falling: ResMut<FallingCount>,
    assets: Res<PieceAssets>,
    mut commands: Commands,
) {
    if game.phase != Phase::AiTurn {
        game.think_timer = 0.0;
        return;
    }
    // Espera a que termine la animación de la jugada anterior.
    if falling.0 > 0 {
        return;
    }

    game.think_timer += time.delta_secs();
    if game.think_timer < 0.35 {
        return;
    }
    game.think_timer = 0.0;

    let player = game.turn;
    if let Some(col) = best_move(&game.board, player, game.difficulty) {
        if let Some(level) = game.apply_move(col) {
            spawn_piece(&mut commands, &assets, player, col, level);
            falling.0 += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_ia_juega_una_columna_legal_y_no_el_centro_al_abrir() {
        let board = Board::new();
        let mv = best_move(&board, Player::X, Difficulty::Medium).unwrap();
        assert!(board.is_legal(mv));
        assert_ne!(mv, CENTER_COLUMN);
    }

    #[test]
    fn la_ia_bloquea_una_linea_vertical() {
        let mut board = Board::new();
        board.place(0, Player::X);
        board.place(1, Player::O);
        board.place(0, Player::X); // X amenaza la columna 0 (z=2)

        let mv = best_move(&board, Player::O, Difficulty::Medium).unwrap();
        assert_eq!(mv, 0, "la IA debe bloquear la columna 0");
    }
}
