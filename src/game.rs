//! Recurso de partida: tablero, turno, fase, dificultad y resultado.

use bevy::prelude::*;

use crate::board::{Board, Player};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Difficulty {
    Easy,
    #[default]
    Medium,
    Hard,
}

impl Difficulty {
    pub fn next(self) -> Self {
        match self {
            Difficulty::Easy => Difficulty::Medium,
            Difficulty::Medium => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Easy,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Win(Player),
    Draw,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Esperando una jugada del jugador humano.
    HumanTurn,
    /// La computadora debe calcular su jugada.
    AiTurn,
    /// Partida terminada.
    Finished,
}

/// Selección hecha en el menú principal.
#[derive(Resource)]
pub struct MenuSelection {
    pub human_first: bool,
    pub difficulty: Difficulty,
}

impl Default for MenuSelection {
    fn default() -> Self {
        MenuSelection {
            human_first: true,
            difficulty: Difficulty::Medium,
        }
    }
}

/// Estado global de la partida.
#[derive(Resource)]
pub struct Game {
    pub board: Board,
    pub human: Player,
    pub ai: Player,
    pub difficulty: Difficulty,
    pub turn: Player,
    pub phase: Phase,
    pub outcome: Option<Outcome>,
    /// Tiempo acumulado de "pensado" de la IA (para dar sensación de espera).
    pub think_timer: f32,
}

impl Default for Game {
    fn default() -> Self {
        Game::new(Player::X, Difficulty::Medium)
    }
}

impl Game {
    /// Crea una partida en la que empieza `first_mover`. El humano es siempre X
    /// y la computadora siempre O.
    pub fn new(first_mover: Player, difficulty: Difficulty) -> Self {
        let human = Player::X;
        let ai = Player::O;
        let turn = first_mover;
        let phase = if turn == human {
            Phase::HumanTurn
        } else {
            Phase::AiTurn
        };
        Game {
            board: Board::new(),
            human,
            ai,
            difficulty,
            turn,
            phase,
            outcome: None,
            think_timer: 0.0,
        }
    }

    pub fn is_human_turn(&self) -> bool {
        self.turn == self.human
    }

    /// Aplica la jugada del jugador actual. Devuelve el nivel ocupado.
    /// Actualiza turno, fase y resultado.
    pub fn apply_move(&mut self, col: usize) -> Option<u8> {
        if self.phase == Phase::Finished {
            return None;
        }
        let level = self.board.place(col, self.turn)?;

        if self.board.is_win(self.turn) {
            self.outcome = Some(Outcome::Win(self.turn));
            self.phase = Phase::Finished;
        } else if self.board.is_full() {
            self.outcome = Some(Outcome::Draw);
            self.phase = Phase::Finished;
        } else {
            self.turn = self.turn.other();
            self.phase = if self.is_human_turn() {
                Phase::HumanTurn
            } else {
                Phase::AiTurn
            };
        }
        Some(level)
    }
}
