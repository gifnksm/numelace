use eframe::egui::{Button, Response, RichText, Sides, Ui, Vec2, widgets};

use crate::{
    action::{Action, ActionRequestQueue, NotesFillScope},
    state::ModalKind,
    ui::{
        icon,
        layout::{ComponentUnits, LayoutScale},
    },
};

#[derive(Debug, Clone)]
pub struct ToolbarViewModel {
    can_undo: bool,
    can_redo: bool,
}

impl ToolbarViewModel {
    #[must_use]
    pub fn new(can_undo: bool, can_redo: bool) -> Self {
        Self { can_undo, can_redo }
    }
}

#[must_use]
pub const fn required_units() -> ComponentUnits {
    ComponentUnits::new(0.0, 1.0)
}

pub fn show(
    ui: &mut Ui,
    vm: &ToolbarViewModel,
    scale: &LayoutScale,
    action_queue: &mut ActionRequestQueue,
) {
    let cell_size = scale.cell_size;
    ui.spacing_mut().item_spacing = Vec2::new(scale.spacing.x, 0.0);
    Sides::new().show(
        ui,
        |ui| {
            ui.horizontal(|ui| {
                if button(ui, icon::ARROW_UNDO, "Undo", vm.can_undo, cell_size).clicked() {
                    action_queue.request(Action::Undo);
                }

                if button(ui, icon::ARROW_REDO, "Redo", vm.can_redo, cell_size).clicked() {
                    action_queue.request(Action::Redo);
                }

                ui.separator();

                if button(ui, icon::PLUS, "New Game", true, cell_size).clicked() {
                    action_queue.request(Action::OpenModal(ModalKind::NewGameConfirm));
                }

                if button(ui, icon::ROTATE_CCW, "Reset Puzzle", true, cell_size).clicked() {
                    action_queue.request(Action::OpenModal(ModalKind::ResetCurrentPuzzleConfirm));
                }

                if button(ui, icon::GEAR_NO_HUB, "Settings", true, cell_size).clicked() {
                    action_queue.request(Action::OpenModal(ModalKind::Settings));
                }

                if button(
                    ui,
                    icon::LETTER_UPPER_A,
                    "Auto-fill notes (all cells)",
                    true,
                    cell_size,
                )
                .clicked()
                {
                    action_queue.request(Action::AutoFillNotes {
                        scope: NotesFillScope::AllCells,
                    });
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
                    action_queue.request(Action::CheckSolvability);
                }
            });
        },
        |ui| {
            widgets::global_theme_preference_switch(ui);
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
