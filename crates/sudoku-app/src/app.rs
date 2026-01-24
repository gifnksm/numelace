//! Sudoku desktop application UI.
//!
//! # Design Notes
//! - Desktop-focused MVP with a 9x9 grid and clear 3x3 boundaries.
//! - Keyboard-driven input (digits, arrows, delete/backspace) with mouse selection.
//! - Status display derived from `Game::is_solved()`.
//!
//! # Future Enhancements
//! - Candidate marks, undo/redo, hints, mistake detection.
//! - Save/load, timer/statistics, and web/WASM support.

use eframe::{
    App, CreationContext, Frame,
    egui::{CentralPanel, Context},
};
use sudoku_core::{Digit, Position};
use sudoku_game::Game;
use sudoku_generator::PuzzleGenerator;
use sudoku_solver::TechniqueSolver;

use crate::ui::{self, Action, MoveDirection};

#[derive(Debug)]
pub struct SudokuApp {
    game: Game,
    selected_cell: Option<Position>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Solved,
}

impl SudokuApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            game: new_game(),
            selected_cell: None,
        }
    }

    fn status(&self) -> GameStatus {
        if self.game.is_solved() {
            GameStatus::Solved
        } else {
            GameStatus::InProgress
        }
    }

    fn new_game(&mut self) {
        self.game = new_game();
        self.selected_cell = None;
    }

    fn set_digit(&mut self, digit: Digit) {
        if let Some(pos) = self.selected_cell {
            let _ = self.game.set_digit(pos, digit);
        }
    }

    fn remove_digit(&mut self) {
        if let Some(pos) = self.selected_cell {
            let _ = self.game.remove_digit(pos);
        }
    }

    fn apply_action(&mut self, action: Action) {
        const DEFAULT_POSITION: Position = Position::new(0, 0);
        match action {
            Action::SelectCell(pos) => {
                self.selected_cell = Some(pos);
            }
            Action::ClearSelection => {
                self.selected_cell = None;
            }
            Action::MoveSelection(move_direction) => {
                let pos = self.selected_cell.get_or_insert(DEFAULT_POSITION);
                let new_pos = match move_direction {
                    MoveDirection::Up => pos.up(),
                    MoveDirection::Down => pos.down(),
                    MoveDirection::Left => pos.left(),
                    MoveDirection::Right => pos.right(),
                };
                if let Some(new_pos) = new_pos {
                    *pos = new_pos;
                }
            }
            Action::SetDigit(digit) => {
                self.set_digit(digit);
            }
            Action::RemoveDigit => {
                self.remove_digit();
            }
            Action::NewGame => {
                self.new_game();
            }
        }
    }
}

fn new_game() -> Game {
    let technique_solver = TechniqueSolver::with_all_techniques();
    let puzzle = PuzzleGenerator::new(&technique_solver).generate();
    Game::new(puzzle)
}

impl App for SudokuApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.input(|i| {
            for action in ui::input::handle_input(i) {
                self.apply_action(action);
            }
        });

        CentralPanel::default().show(ctx, |ui| {
            for action in ui::game_screen::show(ui, &self.game, self.status(), self.selected_cell) {
                self.apply_action(action);
            }
        });
    }
}
