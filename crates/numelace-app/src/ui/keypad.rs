use std::sync::Arc;

use eframe::egui::{
    self, Align2, Button, Color32, FontId, Grid, RichText, Ui, UiBuilder, Vec2, Visuals,
};
use numelace_core::{Digit, containers::Array9, index::DigitSemantics};
use numelace_game::{InputBlockReason, InputOperation};

use crate::{
    action::{Action, ActionRequestQueue, NotesFillScope},
    ui::{
        icon,
        layout::{ComponentUnits, LayoutScale},
    },
};

#[derive(Debug, Clone)]
pub(crate) struct KeypadViewModel {
    digit_states: Array9<DigitKeyState, DigitSemantics>,
    has_removable_input: bool,
    notes_mode: bool,
    auto_fill_capability: Option<Result<InputOperation, InputBlockReason>>,
}

#[derive(Debug, Clone)]
pub(crate) struct DigitKeyState {
    set_digit: Option<Result<InputOperation, InputBlockReason>>,
    toggle_note: Option<Result<InputOperation, InputBlockReason>>,
    decided_count: usize,
}

impl DigitKeyState {
    #[must_use]
    pub(crate) fn new(
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
    #[must_use]
    pub(crate) fn new(
        digit_states: Array9<DigitKeyState, DigitSemantics>,
        has_removable_input: bool,
        notes_mode: bool,
        auto_fill_capability: Option<Result<InputOperation, InputBlockReason>>,
    ) -> Self {
        Self {
            digit_states,
            has_removable_input,
            notes_mode,
            auto_fill_capability,
        }
    }
}

enum ButtonType {
    Digit(Digit),
    ClearCell,
    AutoFillNotes,
    ToggleInputMode,
}

const BUTTON_LAYOUT: [[Option<ButtonType>; 6]; 2] = {
    #[expect(clippy::unnecessary_wraps)]
    const fn d(d: Digit) -> Option<ButtonType> {
        Some(ButtonType::Digit(d))
    }
    #[expect(clippy::unnecessary_wraps)]
    const fn c() -> Option<ButtonType> {
        Some(ButtonType::ClearCell)
    }
    #[expect(clippy::unnecessary_wraps)]
    const fn a() -> Option<ButtonType> {
        Some(ButtonType::AutoFillNotes)
    }
    #[expect(clippy::unnecessary_wraps)]
    const fn t() -> Option<ButtonType> {
        Some(ButtonType::ToggleInputMode)
    }

    #[allow(clippy::enum_glob_use)]
    use Digit::*;
    [
        [d(D1), d(D2), d(D3), d(D4), d(D5), t()],
        [d(D6), d(D7), d(D8), d(D9), c(), a()],
    ]
};

#[expect(clippy::cast_precision_loss)]
const KEYPAD_COLUMNS: f32 = BUTTON_LAYOUT[0].len() as f32;
#[expect(clippy::cast_precision_loss)]
const KEYPAD_ROWS: f32 = BUTTON_LAYOUT.len() as f32;

#[must_use]
pub(crate) fn required_units() -> ComponentUnits {
    ComponentUnits::new(
        KEYPAD_COLUMNS + (KEYPAD_COLUMNS - 1.0) * LayoutScale::SPACING_FACTOR.x,
        KEYPAD_ROWS
            + (KEYPAD_ROWS - 1.0) * LayoutScale::SPACING_FACTOR.y
            + LayoutScale::PADDING_FACTOR.y,
    )
}

#[expect(clippy::cast_precision_loss)]
pub(crate) fn show(
    ui: &mut Ui,
    vm: &KeypadViewModel,
    scale: &LayoutScale,
    action_queue: &mut ActionRequestQueue,
) {
    let style = Arc::clone(ui.style());
    let visuals = &style.visuals;

    let padding = Vec2::new(0.0, scale.padding.y);
    let avail = ui.available_size() - padding;
    let x_buttons = BUTTON_LAYOUT[0].len() as f32;
    let y_buttons = BUTTON_LAYOUT.len() as f32;
    let button_size = scale.cell_size;
    let x_spacing = if x_buttons > 1.0 {
        ((avail.x - button_size * x_buttons) / (x_buttons - 1.0)).max(0.0)
    } else {
        0.0
    };
    let y_spacing = if y_buttons > 1.0 {
        ((avail.y - button_size * y_buttons) / (y_buttons - 1.0)).max(0.0)
    } else {
        0.0
    };
    let rect = ui.available_rect_before_wrap();
    let rect = egui::Rect::from_min_max(rect.min + padding, rect.max);

    let swap_input_mode = ui.input(|i| i.modifiers.command);
    let effective_notes_mode = vm.notes_mode ^ swap_input_mode;
    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        Grid::new(ui.id().with("keypad_grid"))
            .spacing((x_spacing, y_spacing))
            .min_col_width(button_size)
            .min_row_height(button_size)
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
                                if show_clear_button(ui, button_size, vm.has_removable_input) {
                                    action_queue.request(Action::ClearCell);
                                }
                            }
                            Some(ButtonType::AutoFillNotes) => {
                                if show_auto_fill_button(ui, button_size, vm.auto_fill_capability) {
                                    action_queue.request(Action::AutoFillNotes {
                                        scope: NotesFillScope::Cell,
                                    });
                                }
                            }
                            Some(ButtonType::ToggleInputMode) => {
                                if show_toggle_input_mode_button(ui, button_size, vm.notes_mode) {
                                    action_queue.request(Action::ToggleInputMode);
                                }
                            }
                            None => {
                                ui.allocate_space(Vec2::splat(button_size));
                            }
                        }
                    }
                    ui.end_row();
                }
            });
    });
}

struct DigitButtonProps {
    effective_notes_mode: bool,
    capability: Option<Result<InputOperation, InputBlockReason>>,
    digit: Digit,
}

impl DigitButtonProps {
    fn new(state: &DigitKeyState, digit: Digit, effective_notes_mode: bool) -> Self {
        let capability = if effective_notes_mode {
            state.toggle_note
        } else {
            state.set_digit
        };
        Self {
            effective_notes_mode,
            capability,
            digit,
        }
    }

    fn tooltip(&self) -> String {
        let d = self.digit;
        if self.effective_notes_mode {
            match self.capability {
                Some(Ok(InputOperation::Set)) => format!("Add note {d}"),
                Some(Ok(InputOperation::Removed)) => format!("Remove note {d}"),
                Some(Ok(InputOperation::NoOp)) => {
                    format!("Toggle note {d} (blocked by unexpected state)")
                }
                Some(Err(InputBlockReason::Conflict)) => {
                    format!("Add note {d} (blocked by rule violation)")
                }
                Some(Err(InputBlockReason::GivenCell | InputBlockReason::FilledCell)) => {
                    format!("Add note {d} (blocked by filled cell)")
                }
                None => {
                    format!("Toggle note {d} (blocked by no cell selected)")
                }
            }
        } else {
            match self.capability {
                Some(Ok(InputOperation::Set)) => format!("Set digit {d}"),
                Some(Ok(InputOperation::Removed)) => {
                    format!("Set digit {d} (unexpected state)")
                }
                Some(Ok(InputOperation::NoOp)) => {
                    format!("Set digit {d} (already set)")
                }
                Some(Err(InputBlockReason::Conflict)) => {
                    format!("Set digit {d} (blocked by rule violation)")
                }
                Some(Err(InputBlockReason::GivenCell)) => {
                    format!("Set digit {d} (blocked by pre-filled cell)")
                }
                Some(Err(InputBlockReason::FilledCell)) => {
                    format!("Set digit {d} (blocked by unexpected state)")
                }
                None => format!("Set digit {d} (blocked by no cell selected)"),
            }
        }
    }

    fn text_color(&self, visuals: &Visuals) -> Color32 {
        match self.capability {
            Some(Err(InputBlockReason::Conflict)) => visuals.warn_fg_color,
            Some(Ok(_) | Err(InputBlockReason::GivenCell | InputBlockReason::FilledCell))
            | None => visuals.text_color(),
        }
    }

    fn enabled(&self) -> bool {
        match self.capability {
            Some(
                Ok(InputOperation::Set | InputOperation::Removed) | Err(InputBlockReason::Conflict),
            ) => true,
            Some(
                Ok(InputOperation::NoOp)
                | Err(InputBlockReason::GivenCell | InputBlockReason::FilledCell),
            )
            | None => false,
        }
    }

    fn op_icon(&self) -> Option<&'static str> {
        if !self.effective_notes_mode {
            return None;
        }

        match self.capability {
            Some(Ok(InputOperation::Set)) => Some(icon::PENCIL),
            Some(Ok(InputOperation::Removed)) => Some(icon::FOUR_CORNERS),
            Some(
                Ok(InputOperation::NoOp)
                | Err(
                    InputBlockReason::Conflict
                    | InputBlockReason::GivenCell
                    | InputBlockReason::FilledCell,
                ),
            )
            | None => None,
        }
    }
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

    let props = DigitButtonProps::new(state, digit, effective_notes_mode);

    let tooltip = props.tooltip();
    let text = RichText::new(digit.as_str())
        .color(props.text_color(visuals))
        .size(button_size * 0.8);
    let button = Button::new(text).min_size(Vec2::splat(button_size));
    let button = ui
        .add_enabled(props.enabled(), button)
        .on_hover_text(&tooltip)
        .on_disabled_hover_text(&tooltip);
    let clicked = button.clicked();

    ui.painter().text(
        button.rect.right_top() + egui::vec2(-4.0, 2.0),
        Align2::RIGHT_TOP,
        state.decided_count.to_string(),
        FontId::proportional(button_size * 0.25),
        digit_count_color,
    );

    if let Some(op_icon) = props.op_icon() {
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

fn show_clear_button(ui: &mut Ui, button_size: f32, has_removable_input: bool) -> bool {
    let text = RichText::new(icon::GARBAGE_CAN).size(button_size * 0.8);
    let button = Button::new(text).min_size(Vec2::splat(button_size));
    let button = ui
        .add_enabled(has_removable_input, button)
        .on_hover_text("Clear cell (digit and notes)")
        .on_disabled_hover_text("Clear cell (no removable cell selected)");
    button.clicked()
}

fn show_auto_fill_button(
    ui: &mut Ui,
    button_size: f32,
    auto_fill_capability: Option<Result<InputOperation, InputBlockReason>>,
) -> bool {
    let enabled = auto_fill_capability.is_some_and(|res| res.is_ok_and(|op| op.is_set()));
    let text = RichText::new(icon::LETTER_A).size(button_size * 0.8);
    let button = Button::new(text).min_size(Vec2::splat(button_size));
    let button = ui
        .add_enabled(enabled, button)
        .on_hover_text("Auto-fill notes (selected cell)");
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
