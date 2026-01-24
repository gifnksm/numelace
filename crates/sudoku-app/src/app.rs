use std::sync::Arc;

use eframe::{
    CreationContext, Frame,
    egui::{Button, CentralPanel, Context, Grid, InputState, Key, RichText, Vec2},
};
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
        if let Some(pos) = self.selected_cell {
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
                        let _ = self.game.set_digit(pos, digit);
                    } else {
                        let _ = self.game.clear_cell(pos);
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

impl eframe::App for SudokuApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.input(|i| self.handle_input(i));

        CentralPanel::default().show(ctx, |ui| {
            let style = Arc::clone(ui.style());
            let visuals = &style.visuals;

            let avail = ui.available_size();
            let board_size = f32::min(avail.x, avail.y);

            let large_gap = 6.0;
            let small_gap = 2.0;
            let total_gap = large_gap * 2.0 + small_gap * 6.0;
            let cell_size = (board_size - total_gap) / 9.0;

            ui.horizontal_top(|ui| {
                Grid::new(ui.id().with("outer_board"))
                    .spacing((large_gap, large_gap))
                    .min_col_width(cell_size * 3.0 + small_gap * 2.0)
                    .min_row_height(cell_size * 3.0 + small_gap * 2.0)
                    .show(ui, |ui| {
                        for box_row in 0..3 {
                            for box_col in 0..3 {
                                let box_index = box_row * 3 + box_col;
                                Grid::new(ui.id().with(format!("inner_box_{box_row}_{box_col}")))
                                    .spacing((small_gap, small_gap))
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
                                                            .strong()
                                                            .color(visuals.text_color())
                                                    }

                                                    CellState::Filled(digit) => {
                                                        RichText::new(digit.as_str())
                                                            .color(visuals.weak_text_color())
                                                    }
                                                    CellState::Empty => RichText::new(""),
                                                }
                                                .size(cell_size * 0.8);

                                                let mut button = Button::new(text)
                                                    .min_size(Vec2::splat(cell_size));
                                                if self.selected_cell == Some(pos) {
                                                    button = button.fill(visuals.selection.bg_fill);
                                                } else {
                                                    button =
                                                        button.fill(visuals.text_edit_bg_color());
                                                }

                                                let button = ui.add(button);
                                                if button.clicked() {
                                                    self.selected_cell = Some(pos);
                                                }
                                            }
                                            ui.end_row();
                                        }
                                    });
                            }
                            ui.end_row();
                        }
                    });

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
            });
        });
    }
}
