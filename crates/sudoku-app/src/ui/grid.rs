use std::sync::Arc;

use eframe::egui::{Button, Grid, RichText, Stroke, StrokeKind, Ui, Vec2};
use sudoku_core::Position;
use sudoku_game::{CellState, Game};

use crate::ui::Action;

pub fn show(ui: &mut Ui, game: &Game, selected_cell: Option<Position>) -> Vec<Action> {
    let mut actions = vec![];

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
    let selected_digit = selected_cell.and_then(|pos| game.cell(pos).as_digit());

    Grid::new(ui.id().with("outer_board"))
        .spacing((0.0, 0.0))
        .min_col_width(cell_size * 3.0)
        .min_row_height(cell_size * 3.0)
        .show(ui, |ui| {
            for box_row in 0..3 {
                for box_col in 0..3 {
                    let box_index = box_row * 3 + box_col;
                    let grid = Grid::new(ui.id().with(format!("inner_box_{box_row}_{box_col}")))
                        .spacing((0.0, 0.0))
                        .min_col_width(cell_size)
                        .min_row_height(cell_size)
                        .show(ui, |ui| {
                            for cell_row in 0..3 {
                                for cell_col in 0..3 {
                                    let cell_index = cell_row * 3 + cell_col;
                                    let pos = Position::from_box(box_index, cell_index);
                                    let cell = game.cell(pos);
                                    let text = match cell {
                                        CellState::Given(digit) => {
                                            RichText::new(digit.as_str()).color(given_text_color)
                                        }
                                        CellState::Filled(digit) => {
                                            RichText::new(digit.as_str()).color(filled_text_color)
                                        }
                                        CellState::Empty => RichText::new(""),
                                    }
                                    .size(cell_size * 0.8);

                                    let mut button =
                                        Button::new(text).min_size(Vec2::splat(cell_size));
                                    if selected_cell == Some(pos)
                                        || (selected_digit.is_some()
                                            && cell.as_digit() == selected_digit)
                                    {
                                        button = button.fill(selected_bg_color);
                                    } else if selected_cell.is_some_and(|p| {
                                        p.x() == pos.x()
                                            || p.y() == pos.y()
                                            || p.box_index() == pos.box_index()
                                    }) {
                                        button = button.fill(same_home_bg_color);
                                    } else {
                                        button = button.fill(bg_color);
                                    }

                                    let button = ui.add(button);
                                    let border = if selected_cell == Some(pos) {
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
                                        actions.push(Action::SelectCell(pos));
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

    actions
}
