use std::sync::Arc;

use eframe::egui::{self, Align2, Button, FontId, Grid, RichText, Ui, UiBuilder, Vec2, Visuals};
use numelace_core::{Digit, containers::Array9, index::DigitSemantics};
use numelace_game::{InputBlockReason, InputOperation};

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
    set_digit: Option<Result<InputOperation, InputBlockReason>>,
    toggle_note: Option<Result<InputOperation, InputBlockReason>>,
    decided_count: usize,
}

impl DigitKeyState {
    pub fn new(
        digit: Option<Result<InputOperation, InputBlockReason>>,
        note: Option<Result<InputOperation, InputBlockReason>>,
        decided_count: usize,
    ) -> Self {
        Self {
            set_digit: digit,
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
    ClearCell,
    ToggleInputMode,
}

const BUTTON_LAYOUT: [[Option<ButtonType>; 7]; 2] = {
    #[expect(clippy::unnecessary_wraps)]
    const fn d(d: Digit) -> Option<ButtonType> {
        Some(ButtonType::Digit(d))
    }
    #[expect(clippy::unnecessary_wraps)]
    const fn c() -> Option<ButtonType> {
        Some(ButtonType::ClearCell)
    }
    #[expect(clippy::unnecessary_wraps)]
    const fn t() -> Option<ButtonType> {
        Some(ButtonType::ToggleInputMode)
    }

    #[allow(clippy::enum_glob_use)]
    use Digit::*;
    [
        [d(D1), d(D2), d(D3), d(D4), d(D5), None, t()],
        [d(D6), d(D7), d(D8), d(D9), c(), None, None],
    ]
};

#[expect(clippy::cast_precision_loss)]
pub fn show(ui: &mut Ui, vm: &KeypadViewModel, action_queue: &mut ActionRequestQueue) {
    let style = Arc::clone(ui.style());
    let visuals = &style.visuals;

    let x_padding = 5.0;
    let y_padding = 5.0;
    let avail = ui.available_size();
    let x_buttons = BUTTON_LAYOUT[0].len() as f32;
    let y_buttons = BUTTON_LAYOUT.len() as f32;
    let button_size = f32::min(
        (avail.x - (x_buttons - 1.0) * x_padding) / x_buttons,
        (avail.y - (y_buttons - 1.0) * y_padding) / y_buttons,
    );
    // Center the digit block so keys "3" and "8" align with the overall grid center.
    // The block spans 5 columns; its center is column index 2 (0-based).
    let digit_center_offset = button_size * 2.5 + x_padding * 2.0;
    let rect = ui.available_rect_before_wrap();
    let origin_x = rect.center().x - digit_center_offset;
    let rect = rect.intersect(rect.with_min_x(origin_x));

    let swap_input_mode = ui.input(|i| i.modifiers.command);
    let effective_notes_mode = vm.notes_mode ^ swap_input_mode;
    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        Grid::new(ui.id().with("keypad_grid"))
            .spacing((x_padding, y_padding))
            .max_col_width(button_size)
            .show(ui, |ui| {
                for row in BUTTON_LAYOUT {
                    for button_type in row {
                        match button_type {
                            Some(ButtonType::Digit(digit)) => {
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
                            Some(ButtonType::ClearCell) => {
                                if show_remove_button(ui, button_size, vm.has_removable_digit) {
                                    action_queue.request(Action::ClearCell);
                                }
                            }
                            Some(ButtonType::ToggleInputMode) => {
                                if show_toggle_input_mode_button(ui, button_size, vm.notes_mode) {
                                    action_queue.request(Action::ToggleInputMode);
                                }
                            }
                            None => {
                                ui.add_space(button_size);
                            }
                        }
                    }
                    ui.end_row();
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
    let op_icon_color = visuals.text_color();

    let (toggle_capability, tooltip) = if effective_notes_mode {
        (state.toggle_note, "Toggle note")
    } else {
        (state.set_digit, "Set digit")
    };

    let pencil_icon = effective_notes_mode.then_some(icon::PENCIL);
    let four_corners_icon = effective_notes_mode.then_some(icon::FOUR_CORNERS);

    let (op_icon, enabled, text_color) = match toggle_capability {
        Some(Ok(InputOperation::NoOp)) => (None, false, visuals.text_color()),
        Some(Ok(InputOperation::Set)) => (pencil_icon, true, visuals.text_color()),
        Some(Ok(InputOperation::Removed)) => (four_corners_icon, true, visuals.text_color()),
        Some(Err(InputBlockReason::Conflict)) => (None, true, visuals.warn_fg_color),
        Some(Err(InputBlockReason::GivenCell | InputBlockReason::FilledCell)) | None => {
            (None, false, visuals.text_color())
        }
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

    if let Some(op_icon) = op_icon {
        ui.painter().text(
            button.rect.right_bottom() + egui::vec2(-4.0, -2.0),
            Align2::RIGHT_BOTTOM,
            op_icon,
            FontId::proportional(button_size * 0.40),
            op_icon_color,
        );
    }
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

fn show_toggle_input_mode_button(ui: &mut Ui, button_size: f32, notes_mode: bool) -> bool {
    let text = RichText::new(icon::PENCIL).size(button_size * 0.8);
    let button = Button::new(text)
        .selected(notes_mode)
        .min_size(Vec2::splat(button_size));
    let button = ui.add(button).on_hover_text("Toggle Fill/Notes mode");
    button.clicked()
}
