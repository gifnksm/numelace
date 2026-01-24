use sudoku_core::{Digit, Position};

pub mod game_screen;
pub mod grid;
pub mod input;
pub mod keypad;
pub mod sidebar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    SelectCell(Position),
    ClearSelection,
    MoveSelection(MoveDirection),
    SetDigit(Digit),
    RemoveDigit,
    NewGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}
