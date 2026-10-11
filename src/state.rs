//! Estados de alto nivel de la aplicación.

use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    /// Menú principal: elegir quién empieza y la dificultad.
    #[default]
    Menu,
    /// Partida en curso.
    Playing,
    /// La partida terminó (victoria o empate).
    GameOver,
}
