//! Numelace desktop application UI.
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
    egui::{CentralPanel, Context, Visuals},
};
use numelace_core::{Digit, Position};
use numelace_game::Game;
use numelace_generator::PuzzleGenerator;
use numelace_solver::TechniqueSolver;

use crate::ui::{
    self, Action, MoveDirection, game_screen::GameScreenViewModel, grid::GridViewModel,
    keypad::KeypadViewModel, sidebar::SidebarViewModel,
};

#[derive(Debug)]
pub struct NumelaceApp {
    game: Game,
    selected_cell: Option<Position>,
    highlight_config: HighlightConfig,
    theme_config: ThemeConfig,
    show_new_game_confirm_dialogue: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Solved,
}

#[derive(Debug, Clone)]
pub struct HighlightConfig {
    pub same_digit: bool,
    pub rcb_selected: bool,
    pub rcb_same_digit: bool,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            same_digit: true,
            rcb_selected: true,
            rcb_same_digit: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeConfig {
    pub theme: Theme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl NumelaceApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let theme = if cc.egui_ctx.style().visuals.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        };
        Self {
            game: new_game(),
            selected_cell: None,
            highlight_config: HighlightConfig::default(),
            theme_config: ThemeConfig { theme },
            show_new_game_confirm_dialogue: false,
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

    fn apply_action(&mut self, action: Action, ctx: &Context) {
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
            Action::RequestNewGameConfirm => {
                self.show_new_game_confirm_dialogue = true;
            }
            Action::NewGame => {
                self.new_game();
            }
            Action::UpdateHighlightConfig(config) => {
                self.highlight_config = config;
            }
            Action::UpdateThemeConfig(config) => {
                self.theme_config = config;
                match self.theme_config.theme {
                    Theme::Light => {
                        ctx.set_visuals(Visuals::light());
                    }
                    Theme::Dark => {
                        ctx.set_visuals(Visuals::dark());
                    }
                }
            }
        }
    }
}

fn new_game() -> Game {
    let technique_solver = TechniqueSolver::with_all_techniques();
    let puzzle = PuzzleGenerator::new(&technique_solver).generate();
    Game::new(puzzle)
}

impl App for NumelaceApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if !self.show_new_game_confirm_dialogue {
            ctx.input(|i| {
                for action in ui::input::handle_input(i) {
                    self.apply_action(action, ctx);
                }
            });
        }

        let can_set_digit = self
            .selected_cell
            .is_some_and(|pos| self.game.can_set_digit(pos));
        let has_removable_digit = self
            .selected_cell
            .is_some_and(|pos| self.game.has_removable_digit(pos));
        let selected_digit = self
            .selected_cell
            .and_then(|pos| self.game.cell(pos).as_digit());
        let grid_vm = GridViewModel::new(
            &self.game,
            self.selected_cell,
            selected_digit,
            &self.highlight_config,
        );
        let keypad_vm = KeypadViewModel::new(
            can_set_digit,
            has_removable_digit,
            self.game.decided_digit_count(),
        );
        let sidebar_vm =
            SidebarViewModel::new(self.status(), &self.highlight_config, &self.theme_config);
        let game_screen_vm = GameScreenViewModel::new(grid_vm, keypad_vm, sidebar_vm);

        let mut actions = vec![];
        CentralPanel::default().show(ctx, |ui| {
            actions = ui::game_screen::show(ui, &game_screen_vm);
            if self.show_new_game_confirm_dialogue {
                actions.extend(ui::dialogs::show_new_game_confirm(
                    ui,
                    &mut self.show_new_game_confirm_dialogue,
                ));
            }
        });

        for action in actions {
            self.apply_action(action, ctx);
        }
    }
}
