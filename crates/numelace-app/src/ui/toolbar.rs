use eframe::egui::{Button, Id, Popup, Response, RichText, ThemePreference, Ui, Vec2};
use numelace_game::{InputBlockReason, InputOperation};

use crate::{
    action::{
        ActionRequestQueue, BoardMutationAction, FlowAction, HistoryAction, ModalRequest,
        NotesFillScope, UiAction,
    },
    ui::{
        icon,
        layout::{ComponentUnits, LayoutScale},
    },
};

#[derive(Debug, Clone)]
pub(crate) struct ToolbarViewModel {
    can_undo: bool,
    can_redo: bool,
    selected_cell_auto_fill_capability: Option<Result<InputOperation, InputBlockReason>>,
}

impl ToolbarViewModel {
    #[must_use]
    pub(crate) fn new(
        can_undo: bool,
        can_redo: bool,
        selected_cell_auto_fill_capability: Option<Result<InputOperation, InputBlockReason>>,
    ) -> Self {
        Self {
            can_undo,
            can_redo,
            selected_cell_auto_fill_capability,
        }
    }
}

#[must_use]
pub(crate) const fn required_units() -> ComponentUnits {
    ComponentUnits::new(0.0, 1.0)
}

pub(crate) fn show(
    ui: &mut Ui,
    vm: &ToolbarViewModel,
    scale: &LayoutScale,
    action_queue: &mut ActionRequestQueue,
) {
    let cell_size = scale.cell_size;
    ui.spacing_mut().item_spacing = Vec2::new(scale.spacing.x, 0.0);
    ui.horizontal(|ui| {
        if button(ui, icon::ARROW_UNDO, "Undo", vm.can_undo, cell_size).clicked() {
            action_queue.request(HistoryAction::Undo.into());
        }

        if button(ui, icon::ARROW_REDO, "Redo", vm.can_redo, cell_size).clicked() {
            action_queue.request(HistoryAction::Redo.into());
        }

        if button(
            ui,
            icon::SEARCH_RIGHT,
            "Check whether the current board still has a solution.",
            true,
            cell_size,
        )
        .clicked()
        {
            action_queue.request(FlowAction::CheckSolvability.into());
        }

        if button(
            ui,
            icon::LIGHTBULB,
            "Get a hint (stage 1).",
            true,
            cell_size,
        )
        .clicked()
        {
            action_queue.request(FlowAction::Hint.into());
        }

        ui.separator();

        if button(ui, icon::PLUS, "New Game", true, cell_size).clicked() {
            action_queue.request(FlowAction::StartNewGame.into());
        }

        if button(ui, icon::ROTATE_CCW, "Reset Inputs", true, cell_size).clicked() {
            action_queue.request(FlowAction::ResetInputs.into());
        }

        if button(ui, icon::GEAR_NO_HUB, "Settings", true, cell_size).clicked() {
            action_queue.request(UiAction::OpenModal(ModalRequest::Settings).into());
        }

        ui.separator();

        let response = button(ui, icon::MENU, "More", true, cell_size);
        Popup::menu(&response)
            .id(Id::new("toolbar_more_menu"))
            .show(|ui| show_menu(ui, vm, cell_size, action_queue));
    });
}

fn show_menu(
    ui: &mut Ui,
    vm: &ToolbarViewModel,
    cell_size: f32,
    action_queue: &mut ActionRequestQueue,
) {
    if menu_button(
        ui,
        &format!("{} Auto-fill notes (all cells)", icon::LETTER_UPPER_A),
        "Automatically fill in notes for all cells based on the current board state.",
        true,
        cell_size,
    )
    .clicked()
    {
        action_queue.request(
            BoardMutationAction::AutoFillNotes {
                scope: NotesFillScope::AllCells,
            }
            .into(),
        );
    }
    if menu_button(
        ui,
        &format!("{} Auto-fill notes (empty cells)", icon::LETTER_UPPER_A),
        "Automatically fill in notes for empty cells based on the current board state",
        true,
        cell_size,
    )
    .clicked()
    {
        action_queue.request(
            BoardMutationAction::AutoFillNotes {
                scope: NotesFillScope::EmptyCells,
            }
            .into(),
        );
    }
    if menu_button(
        ui,
        &format!("{} Auto-fill notes (selected cell)", icon::LETTER_A),
        "Automatically fill in notes for selected cell based on the current board state",
        vm.selected_cell_auto_fill_capability
            .is_some_and(|res| res.is_ok_and(|op| op.is_set())),
        cell_size,
    )
    .clicked()
    {
        action_queue.request(
            BoardMutationAction::AutoFillNotes {
                scope: NotesFillScope::SelectedCell,
            }
            .into(),
        );
    }

    ui.separator();

    ui.menu_button(
        menu_text(&format!("{} Appearance", icon::PALETTE), cell_size),
        |ui| {
            let mut theme_preference = ui.ctx().options(|opt| opt.theme_preference);
            ui.radio_value(
                &mut theme_preference,
                ThemePreference::System,
                menu_text(&format!("{} System", icon::LAPTOP), cell_size),
            )
            .on_hover_text("Follow the system theme preference.");
            ui.radio_value(
                &mut theme_preference,
                ThemePreference::Dark,
                menu_text("🌙 Dark", cell_size),
            )
            .on_hover_text("Use dark mode theme");
            ui.radio_value(
                &mut theme_preference,
                ThemePreference::Light,
                menu_text(&format!("{} Light", icon::SUN), cell_size),
            )
            .on_hover_text("Use light mode theme");
            ui.ctx().set_theme(theme_preference);
        },
    );
}

fn button(ui: &mut Ui, label: &str, hover_text: &str, enabled: bool, cell_size: f32) -> Response {
    let text_size = cell_size * 0.8;
    ui.add_enabled(
        enabled,
        Button::new(RichText::new(label).size(text_size)).min_size(Vec2::splat(cell_size)),
    )
    .on_hover_text(hover_text)
}

fn menu_text(text: &str, cell_size: f32) -> RichText {
    RichText::new(text).size(cell_size * 0.3)
}

fn menu_button(
    ui: &mut Ui,
    label: &str,
    hover_text: &str,
    enabled: bool,
    cell_size: f32,
) -> Response {
    ui.add_enabled(enabled, Button::new(menu_text(label, cell_size)))
        .on_hover_text(hover_text)
}
