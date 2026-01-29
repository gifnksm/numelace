use std::sync::Arc;

use eframe::egui::{self, Align2, Button, FontId, Grid, Layout, RichText, Ui, Vec2, Visuals};
use numelace_core::{Digit, containers::Array9, index::DigitSemantics};
use numelace_game::ToggleCapability;

use crate::{
    action::{Action, ActionRequestQueue},
    ui::icon,
};

#[derive(Debug, Clone)]
pub struct KeypadViewModel {
    digit_states: Array9<DigitKeyState, DigitSemantics>,
    has_removable_digit: bool,
    notes_mode: bool,
}

#[derive(Debug, Clone)]
pub struct DigitKeyState {
    toggle_digit: Option<ToggleCapability>,
    toggle_note: Option<ToggleCapability>,
    decided_count: usize,
}

impl DigitKeyState {
    pub fn new(
        digit: Option<ToggleCapability>,
        note: Option<ToggleCapability>,
        decided_count: usize,
    ) -> Self {
        Self {
            toggle_digit: digit,
            toggle_note: note,
            decided_count,
        }
    }
}

impl KeypadViewModel {
    pub fn new(
        digit_states: Array9<DigitKeyState, DigitSemantics>,
        has_removable_digit: bool,
        notes_mode: bool,
    ) -> Self {
        Self {
            digit_states,
            has_removable_digit,
            notes_mode,
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

pub fn show(ui: &mut Ui, vm: &KeypadViewModel, action_queue: &mut ActionRequestQueue) {
    let style = Arc::clone(ui.style());
    let visuals = &style.visuals;

    let x_padding = 5.0;
    let y_padding = 5.0;
    let avail = ui.available_size();
    let button_size = f32::min(
        (avail.x - 4.0 * x_padding) / 5.0,
        (avail.y - y_padding) / 2.0,
    );
    let buttons_width = button_size * 5.0 + x_padding * 4.0;
    let left_pad = (avail.x - buttons_width) / 2.0;
    let swap_input_mode = ui.input(|i| i.modifiers.command);
    let effective_notes_mode = vm.notes_mode ^ swap_input_mode;

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
                                    if show_digit_button(
                                        ui,
                                        digit,
                                        button_size,
                                        effective_notes_mode,
                                        &vm.digit_states[digit],
                                        visuals,
                                    ) {
                                        action_queue.request(Action::RequestDigit {
                                            digit,
                                            swap: swap_input_mode,
                                        });
                                    }
                                }
                                ButtonType::RemoveDigit => {
                                    if show_remove_button(ui, button_size, vm.has_removable_digit) {
                                        action_queue.request(Action::ClearCell);
                                    }
                                }
                            }
                        }
                        ui.end_row();
                    }
                });
        });

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let swap_marker = if swap_input_mode { "^" } else { "" };
                ui.label(RichText::new("Effective Mode:").size(button_size * 0.2));
                if effective_notes_mode {
                    ui.label(
                        RichText::new(format!("{swap_marker}Notes"))
                            .size(button_size * 0.2)
                            .background_color(visuals.selection.bg_fill),
                    );
                } else {
                    ui.label(RichText::new(format!("{swap_marker}Fill")).size(button_size * 0.2));
                }
            });
            let mut notes_mode = vm.notes_mode;
            if ui
                .checkbox(
                    &mut notes_mode,
                    RichText::new("Fill/Notes").size(button_size * 0.2),
                )
                .changed()
            {
                action_queue.request(Action::ToggleInputMode);
            }
        });
    });
}

fn show_digit_button(
    ui: &mut Ui,
    digit: Digit,
    button_size: f32,
    effective_notes_mode: bool,
    state: &DigitKeyState,
    visuals: &Visuals,
) -> bool {
    let digit_count_color = visuals.text_color();

    let (toggle_capability, tooltip) = if effective_notes_mode {
        (state.toggle_note, "Toggle note")
    } else {
        (state.toggle_digit, "Toggle digit")
    };

    let (enabled, text_color) = match toggle_capability {
        Some(ToggleCapability::BlockedByConflict) => (true, visuals.warn_fg_color),
        Some(ToggleCapability::Allowed) => (true, visuals.text_color()),
        Some(ToggleCapability::BlockedByGivenCell | ToggleCapability::BlockedByFilledCell)
        | None => (false, visuals.text_color()),
    };

    let text = RichText::new(digit.as_str())
        .color(text_color)
        .size(button_size * 0.8);
    let button = Button::new(text).min_size(Vec2::splat(button_size));
    let button = ui.add_enabled(enabled, button).on_hover_text(tooltip);
    let clicked = button.clicked();
    ui.painter().text(
        button.rect.right_top() + egui::vec2(-4.0, 2.0),
        Align2::RIGHT_TOP,
        state.decided_count.to_string(),
        FontId::proportional(button_size * 0.25),
        digit_count_color,
    );
    clicked
}

fn show_remove_button(ui: &mut Ui, button_size: f32, has_removable_digit: bool) -> bool {
    let text = RichText::new(icon::GARBAGE_CAN).size(button_size * 0.8);
    let button = Button::new(text).min_size(Vec2::splat(button_size));
    let button = ui
        .add_enabled(has_removable_digit, button)
        .on_hover_text("Clear cell (digit + notes)");
    button.clicked()
}
