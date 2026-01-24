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
use std::sync::Arc;

use eframe::{
    App, CreationContext, Frame,
    egui::{
        self, Align2, Button, CentralPanel, Context, FontId, Grid, InputState, Key, RichText,
        Stroke, StrokeKind, Ui, Vec2,
    },
};
use egui_extras::{Size, StripBuilder};
use sudoku_core::{Digit, Position};
use sudoku_game::{CellState, Game};
use sudoku_generator::PuzzleGenerator;
use sudoku_solver::TechniqueSolver;

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

    fn handle_input(&mut self, i: &InputState) {
        const DEFAULT_POSITION: Position = Position::new(0, 0);
        if (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::N) {
            self.new_game();
        }
        if i.key_pressed(Key::ArrowUp) {
            let pos = self.selected_cell.get_or_insert(DEFAULT_POSITION);
            if let Some(p) = pos.up() {
                *pos = p;
            }
        }
        if i.key_pressed(Key::ArrowDown) {
            let pos = self.selected_cell.get_or_insert(DEFAULT_POSITION);
            if let Some(p) = pos.down() {
                *pos = p;
            }
        }
        if i.key_pressed(Key::ArrowLeft) {
            let pos = self.selected_cell.get_or_insert(DEFAULT_POSITION);
            if let Some(p) = pos.left() {
                *pos = p;
            }
        }
        if i.key_pressed(Key::ArrowRight) {
            let pos = self.selected_cell.get_or_insert(DEFAULT_POSITION);
            if let Some(p) = pos.right() {
                *pos = p;
            }
        }
        if i.key_pressed(Key::Escape) {
            self.selected_cell = None;
        }

        let pairs = [
            (Key::Delete, None),
            (Key::Backspace, None),
            (Key::Num1, Some(Digit::D1)),
            (Key::Num2, Some(Digit::D2)),
            (Key::Num3, Some(Digit::D3)),
            (Key::Num4, Some(Digit::D4)),
            (Key::Num5, Some(Digit::D5)),
            (Key::Num6, Some(Digit::D6)),
            (Key::Num7, Some(Digit::D7)),
            (Key::Num8, Some(Digit::D8)),
            (Key::Num9, Some(Digit::D9)),
        ];
        for (key, digit) in pairs {
            if i.key_pressed(key) {
                if let Some(digit) = digit {
                    self.set_digit(digit);
                } else {
                    self.remove_digit();
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

impl App for SudokuApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.input(|i| self.handle_input(i));

        CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::relative(0.75))
                .size(Size::relative(0.25))
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        StripBuilder::new(ui)
                            .size(Size::relative(9.0 / (9.0 + 2.0)))
                            .size(Size::relative(2.0 / (9.0 + 2.0)))
                            .vertical(|mut strip| {
                                strip.cell(|ui| {
                                    self.draw_grid(ui);
                                });
                                strip.cell(|ui| {
                                    self.draw_keypad(ui);
                                });
                            });
                    });
                    strip.cell(|ui| {
                        self.draw_sidebar(ui);
                    });
                });
        });
    }
}

impl SudokuApp {
    fn draw_grid(&mut self, ui: &mut Ui) {
        let style = Arc::clone(ui.style());
        let visuals = &style.visuals;
        let border_color = visuals.widgets.inactive.fg_stroke.color;
        let given_text_color = visuals.strong_text_color();
        let filled_text_color = visuals.text_color();
        let selected_bg_color = visuals.selection.bg_fill;
        let same_home_bg_color = visuals.widgets.hovered.bg_fill;
        let bg_color = visuals.text_edit_bg_color();

        let thin_border = Stroke::new(1.0, border_color);
        let thick_border = Stroke::new(3.0, border_color);
        let selected_border = Stroke::new(6.0, border_color);

        let board_size = ui.available_size().min_elem();
        let cell_size = board_size / 9.0;
        let selected_digit = self
            .selected_cell
            .and_then(|pos| self.game.cell(pos).as_digit());

        Grid::new(ui.id().with("outer_board"))
            .spacing((0.0, 0.0))
            .min_col_width(cell_size * 3.0)
            .min_row_height(cell_size * 3.0)
            .show(ui, |ui| {
                for box_row in 0..3 {
                    for box_col in 0..3 {
                        let box_index = box_row * 3 + box_col;
                        let grid =
                            Grid::new(ui.id().with(format!("inner_box_{box_row}_{box_col}")))
                                .spacing((0.0, 0.0))
                                .min_col_width(cell_size)
                                .min_row_height(cell_size)
                                .show(ui, |ui| {
                                    for cell_row in 0..3 {
                                        for cell_col in 0..3 {
                                            let cell_index = cell_row * 3 + cell_col;
                                            let pos = Position::from_box(box_index, cell_index);
                                            let cell = self.game.cell(pos);
                                            let text = match cell {
                                                CellState::Given(digit) => {
                                                    RichText::new(digit.as_str())
                                                        .color(given_text_color)
                                                }
                                                CellState::Filled(digit) => {
                                                    RichText::new(digit.as_str())
                                                        .color(filled_text_color)
                                                }
                                                CellState::Empty => RichText::new(""),
                                            }
                                            .size(cell_size * 0.8);

                                            let mut button =
                                                Button::new(text).min_size(Vec2::splat(cell_size));
                                            if self.selected_cell == Some(pos)
                                                || (selected_digit.is_some()
                                                    && cell.as_digit() == selected_digit)
                                            {
                                                button = button.fill(selected_bg_color);
                                            } else if self.selected_cell.is_some_and(|p| {
                                                p.x() == pos.x()
                                                    || p.y() == pos.y()
                                                    || p.box_index() == pos.box_index()
                                            }) {
                                                button = button.fill(same_home_bg_color);
                                            } else {
                                                button = button.fill(bg_color);
                                            }

                                            let button = ui.add(button);
                                            let border = if self.selected_cell == Some(pos) {
                                                selected_border
                                            } else {
                                                thin_border
                                            };
                                            ui.painter().rect_stroke(
                                                button.rect,
                                                0.0,
                                                border,
                                                StrokeKind::Inside,
                                            );
                                            if button.clicked() {
                                                self.selected_cell = Some(pos);
                                            }
                                        }
                                        ui.end_row();
                                    }
                                });
                        ui.painter().rect_stroke(
                            grid.response.rect,
                            0.0,
                            thick_border,
                            StrokeKind::Inside,
                        );
                    }
                    ui.end_row();
                }
            });
    }

    fn draw_keypad(&mut self, ui: &mut Ui) {
        #[allow(clippy::enum_glob_use)]
        use Digit::*;
        enum ButtonType {
            Digit(Digit),
            RemoveDigit,
        }
        fn d(d: Digit) -> ButtonType {
            ButtonType::Digit(d)
        }
        fn r() -> ButtonType {
            ButtonType::RemoveDigit
        }

        let style = Arc::clone(ui.style());
        let visuals = &style.visuals;
        let digit_count_color = visuals.text_color();

        let layout = [
            [d(D1), d(D2), d(D3), d(D4), d(D5)],
            [d(D6), d(D7), d(D8), d(D9), r()],
        ];

        let x_padding = 5.0;
        let y_padding = 5.0;
        let avail = ui.available_size();
        let button_size = f32::min(
            (avail.x - 4.0 * x_padding) / 5.0,
            (avail.y - y_padding) / 2.0,
        );
        let counts = self.game.decided_digit_count();

        let button_enabled = self
            .selected_cell
            .is_some_and(|pos| !self.game.cell(pos).is_given());

        Grid::new(ui.id().with("keypad_grid"))
            .spacing((x_padding, y_padding))
            .show(ui, |ui| {
                for row in &layout {
                    for button_type in row {
                        match button_type {
                            ButtonType::Digit(digit) => {
                                let text = RichText::new(digit.as_str()).size(button_size * 0.8);
                                let button = Button::new(text).min_size(Vec2::splat(button_size));
                                let button = ui
                                    .add_enabled(button_enabled, button)
                                    .on_hover_text("Set digit");
                                if button.clicked() {
                                    self.set_digit(*digit);
                                }
                                ui.painter().text(
                                    button.rect.right_top() + egui::vec2(-4.0, 2.0),
                                    Align2::RIGHT_TOP,
                                    counts[*digit].to_string(),
                                    FontId::proportional(button_size * 0.25),
                                    digit_count_color,
                                );
                            }
                            ButtonType::RemoveDigit => {
                                let text = RichText::new("X").size(button_size * 0.8);
                                let button = Button::new(text).min_size(Vec2::splat(button_size));
                                let button = ui
                                    .add_enabled(button_enabled, button)
                                    .on_hover_text("Remove digit");
                                if button.clicked() {
                                    self.remove_digit();
                                }
                            }
                        }
                    }
                    ui.end_row();
                }
            });
    }

    fn draw_sidebar(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            let text = match self.status() {
                GameStatus::InProgress => "Game in progress",
                GameStatus::Solved => "Congratulations! You solved the puzzle!",
            };
            ui.label(RichText::new(text).size(20.0));
            if ui.button(RichText::new("New Game").size(20.0)).clicked() {
                self.new_game();
            }
        });
    }
}
