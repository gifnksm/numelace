use numelace_core::{Digit, Position};

use crate::app::{HighlightConfig, ThemeConfig};

pub mod dialogs;
pub mod game_screen;
pub mod grid;
pub mod input;
pub mod keypad;
pub mod sidebar;

#[derive(Debug, Clone)]
pub enum Action {
    SelectCell(Position),
    ClearSelection,
    MoveSelection(MoveDirection),
    SetDigit(Digit),
    RemoveDigit,
    RequestNewGameConfirm,
    NewGame,
    UpdateHighlightConfig(HighlightConfig),
    UpdateThemeConfig(ThemeConfig),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}
