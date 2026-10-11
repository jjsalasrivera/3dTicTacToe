//! Interfaz de usuario: menú principal, HUD de partida y pantalla de fin.

use bevy::prelude::*;

use crate::board::Player;
use crate::game::{Game, MenuSelection, Outcome, Phase};
use crate::pieces::{FallingCount, Piece};
use crate::state::AppState;

const MENU_BUTTON: Color = Color::srgb(0.16, 0.17, 0.22);
const MENU_BUTTON_HOVER: Color = Color::srgb(0.26, 0.28, 0.36);
const MENU_BUTTON_PRESS: Color = Color::srgb(0.30, 0.45, 0.30);

#[derive(Component)]
struct MenuUi;
#[derive(Component)]
struct HudUi;
#[derive(Component)]
struct GameOverUi;

#[derive(Component)]
struct HudStatusText;

#[derive(Component, Clone, Copy)]
enum MenuAction {
    StartHuman,
    StartAi,
    CycleDifficulty,
}

#[derive(Component)]
struct DifficultyLabel;

#[derive(Component)]
struct BackToMenu;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(OnExit(AppState::Menu), cleanup::<MenuUi>)
            .add_systems(Update, menu_system.run_if(in_state(AppState::Menu)))
            .add_systems(OnEnter(AppState::Playing), (start_game, setup_hud))
            .add_systems(OnExit(AppState::Playing), cleanup::<HudUi>)
            .add_systems(
                Update,
                (update_hud, check_game_over).run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnEnter(AppState::GameOver), setup_game_over)
            .add_systems(OnExit(AppState::GameOver), cleanup::<GameOverUi>)
            .add_systems(Update, game_over_system.run_if(in_state(AppState::GameOver)));
    }
}

fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// Menú principal
// ---------------------------------------------------------------------------

fn setup_menu(mut commands: Commands) {
    let root = commands
        .spawn((
            MenuUi,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(16),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.82)),
        ))
        .id();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Text::new("Tres en Raya 3D"),
            TextFont {
                font_size: FontSize::Px(58.0),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.82, 0.45)),
        ));
        parent.spawn((
            Text::new("Alinea 3 fichas en cualquier eje o diagonal del cubo 3x3x3."),
            TextFont {
                font_size: FontSize::Px(18.0),
                ..default()
            },
            TextColor(Color::srgb(0.82, 0.84, 0.9)),
        ));
        parent.spawn((
            Text::new("Tú eres X (rojo) · la computadora es O (azul)"),
            TextFont {
                font_size: FontSize::Px(16.0),
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.72, 0.8)),
        ));
        parent.spawn(menu_button("Empiezo yo", MenuAction::StartHuman));
        parent.spawn(menu_button("Empieza la computadora", MenuAction::StartAi));
        parent.spawn(difficulty_button());
        parent.spawn((
            Text::new(
                "Arrastra con el botón izquierdo para girar el tablero · rueda para acercar",
            ),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.62, 0.7)),
        ));
    });
}

fn menu_button(label: &str, action: MenuAction) -> impl Bundle {
    (
        Button,
        action,
        Node {
            width: px(340),
            height: px(56),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BorderColor::all(Color::srgb(0.55, 0.45, 0.25)),
        BackgroundColor(MENU_BUTTON),
        children![(
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(24.0),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.95, 0.95)),
        )],
    )
}

fn difficulty_button() -> impl Bundle {
    (
        Button,
        MenuAction::CycleDifficulty,
        Node {
            width: px(340),
            height: px(56),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BorderColor::all(Color::srgb(0.55, 0.45, 0.25)),
        BackgroundColor(MENU_BUTTON),
        children![(
            DifficultyLabel,
            Text::new("Dificultad: Medio"),
            TextFont {
                font_size: FontSize::Px(24.0),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.95, 0.95)),
        )],
    )
}

fn menu_system(
    mut interaction: Query<(&Interaction, &MenuAction, &mut BackgroundColor), Changed<Interaction>>,
    mut selection: ResMut<MenuSelection>,
    mut next: ResMut<NextState<AppState>>,
    mut labels: Query<&mut Text, With<DifficultyLabel>>,
) {
    for (interaction, action, mut background) in &mut interaction {
        match *interaction {
            Interaction::Pressed => {
                *background = MENU_BUTTON_PRESS.into();
                match action {
                    MenuAction::StartHuman => {
                        selection.human_first = true;
                        next.set(AppState::Playing);
                    }
                    MenuAction::StartAi => {
                        selection.human_first = false;
                        next.set(AppState::Playing);
                    }
                    MenuAction::CycleDifficulty => {
                        selection.difficulty = selection.difficulty.next();
                        for mut text in &mut labels {
                            **text = format!("Dificultad: {}", selection.difficulty.label());
                        }
                    }
                }
            }
            Interaction::Hovered => *background = MENU_BUTTON_HOVER.into(),
            Interaction::None => *background = MENU_BUTTON.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Partida
// ---------------------------------------------------------------------------

fn start_game(
    mut commands: Commands,
    selection: Res<MenuSelection>,
    mut game: ResMut<Game>,
    mut falling: ResMut<FallingCount>,
    pieces: Query<Entity, With<Piece>>,
) {
    for entity in &pieces {
        commands.entity(entity).despawn();
    }
    falling.0 = 0;
    let first = if selection.human_first {
        Player::X
    } else {
        Player::O
    };
    *game = Game::new(first, selection.difficulty);
}

fn setup_hud(mut commands: Commands) {
    let root = commands
        .spawn((
            HudUi,
            Node {
                position_type: PositionType::Absolute,
                top: px(14),
                left: px(14),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
        ))
        .id();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            HudStatusText,
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(26.0),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        parent.spawn((
            Text::new("Clic para colocar · arrastra para girar · rueda para zoom"),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::srgb(0.78, 0.8, 0.88)),
        ));
    });
}

fn update_hud(
    game: Res<Game>,
    falling: Res<FallingCount>,
    mut query: Query<&mut Text, With<HudStatusText>>,
) {
    let message = if falling.0 > 0 {
        format!("Colocando {}…", game.turn.name())
    } else {
        match game.outcome {
            Some(Outcome::Win(player)) if player == game.ai => "Ganó la computadora".to_string(),
            Some(Outcome::Win(_)) => "¡Ganaste!".to_string(),
            Some(Outcome::Draw) => "¡Empate!".to_string(),
            None => match game.phase {
                Phase::AiTurn => "Turno de la computadora…".to_string(),
                _ if game.board.moves == 0 => {
                    "Tu turno (X). No puedes empezar en el centro.".to_string()
                }
                _ => "Tu turno (X)".to_string(),
            },
        }
    };

    for mut text in &mut query {
        **text = message.clone();
    }
}

fn check_game_over(
    game: Res<Game>,
    falling: Res<FallingCount>,
    mut next: ResMut<NextState<AppState>>,
) {
    if game.outcome.is_some() && falling.0 == 0 {
        next.set(AppState::GameOver);
    }
}

// ---------------------------------------------------------------------------
// Fin de partida
// ---------------------------------------------------------------------------

fn setup_game_over(mut commands: Commands, game: Res<Game>) {
    let title = match game.outcome {
        Some(Outcome::Win(player)) if player == game.human => "¡Ganaste!",
        Some(Outcome::Win(_)) => "Ganó la computadora",
        Some(Outcome::Draw) => "Empate",
        None => "",
    };

    let root = commands
        .spawn((
            GameOverUi,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(20),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        ))
        .id();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Text::new(title.to_string()),
            TextFont {
                font_size: FontSize::Px(52.0),
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.88, 0.5)),
        ));
        parent.spawn(game_over_button());
    });
}

fn game_over_button() -> impl Bundle {
    (
        Button,
        BackToMenu,
        Node {
            width: px(300),
            height: px(54),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BorderColor::all(Color::srgb(0.55, 0.45, 0.25)),
        BackgroundColor(MENU_BUTTON),
        children![(
            Text::new("Volver al menú"),
            TextFont {
                font_size: FontSize::Px(22.0),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.95, 0.95)),
        )],
    )
}

#[allow(clippy::type_complexity)]
fn game_over_system(
    mut interaction: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<BackToMenu>),
    >,
    mut next: ResMut<NextState<AppState>>,
) {
    for (interaction, mut background) in &mut interaction {
        match *interaction {
            Interaction::Pressed => next.set(AppState::Menu),
            Interaction::Hovered => *background = MENU_BUTTON_HOVER.into(),
            Interaction::None => *background = MENU_BUTTON.into(),
        }
    }
}
