use std::sync::Arc;

use eframe::egui::{self, Align2, Button, FontId, Grid, Layout, RichText, Ui, Vec2};
use numelace_core::{Digit, containers::Array9, index::DigitSemantics};

use crate::ui::Action;

#[derive(Debug, Clone)]
pub struct KeypadViewModel {
    can_set_digit: bool,
    has_removable_digit: bool,
    decided_digit_count: Array9<usize, DigitSemantics>,
}

impl KeypadViewModel {
    pub fn new(
        can_set_digit: bool,
        has_removable_digit: bool,
        decided_digit_count: Array9<usize, DigitSemantics>,
    ) -> Self {
        Self {
            can_set_digit,
            has_removable_digit,
            decided_digit_count,
        }
    }
}

enum ButtonType {
    Digit(Digit),
    RemoveDigit,
}

const BUTTON_LAYOUT: [[ButtonType; 5]; 2] = {
    const fn d(d: Digit) -> ButtonType {
        ButtonType::Digit(d)
    }
    const fn r() -> ButtonType {
        ButtonType::RemoveDigit
    }

    #[allow(clippy::enum_glob_use)]
    use Digit::*;
    [
        [d(D1), d(D2), d(D3), d(D4), d(D5)],
        [d(D6), d(D7), d(D8), d(D9), r()],
    ]
};

pub fn show(ui: &mut Ui, vm: &KeypadViewModel) -> Vec<Action> {
    let mut actions = vec![];

    let style = Arc::clone(ui.style());
    let visuals = &style.visuals;
    let digit_count_color = visuals.text_color();

    let x_padding = 5.0;
    let y_padding = 5.0;
    let avail = ui.available_size();
    let button_size = f32::min(
        (avail.x - 4.0 * x_padding) / 5.0,
        (avail.y - y_padding) / 2.0,
    );
    let buttons_width = button_size * 5.0 + x_padding * 4.0;
    let left_pad = (avail.x - buttons_width) / 2.0;
    ui.horizontal(|ui| {
        ui.add_space(left_pad);
        ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            Grid::new(ui.id().with("keypad_grid"))
                .spacing((x_padding, y_padding))
                .max_col_width(button_size)
                .show(ui, |ui| {
                    for row in BUTTON_LAYOUT {
                        for button_type in row {
                            match button_type {
                                ButtonType::Digit(digit) => {
                                    let text =
                                        RichText::new(digit.as_str()).size(button_size * 0.8);
                                    let button =
                                        Button::new(text).min_size(Vec2::splat(button_size));
                                    let button = ui
                                        .add_enabled(vm.can_set_digit, button)
                                        .on_hover_text("Set digit");
                                    if button.clicked() {
                                        actions.push(Action::SetDigit(digit));
                                    }
                                    ui.painter().text(
                                        button.rect.right_top() + egui::vec2(-4.0, 2.0),
                                        Align2::RIGHT_TOP,
                                        vm.decided_digit_count[digit].to_string(),
                                        FontId::proportional(button_size * 0.25),
                                        digit_count_color,
                                    );
                                }
                                ButtonType::RemoveDigit => {
                                    let text = RichText::new("X").size(button_size * 0.8);
                                    let button =
                                        Button::new(text).min_size(Vec2::splat(button_size));
                                    let button = ui
                                        .add_enabled(vm.has_removable_digit, button)
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
        });
    });
    actions
}
