use std::sync::Arc;

use eframe::egui::{self, Align2, Button, FontId, Grid, RichText, Ui, Vec2};
use sudoku_core::{Digit, Position};
use sudoku_game::Game;

use crate::ui::Action;

pub fn show(ui: &mut Ui, game: &Game, selected_cell: Option<Position>) -> Vec<Action> {
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

    let mut actions = vec![];

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
    let counts = game.decided_digit_count();

    let button_enabled = selected_cell.is_some_and(|pos| !game.cell(pos).is_given());

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
                                actions.push(Action::SetDigit(*digit));
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
                                actions.push(Action::RemoveDigit);
                            }
                        }
                    }
                }
                ui.end_row();
            }
        });

    actions
}
