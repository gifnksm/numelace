use eframe::egui::{Button, Response, RichText, Ui};

use crate::{
    action::{Action, ActionRequestQueue},
    ui::icon,
};

#[derive(Debug, Clone)]
pub struct ToolbarViewModel {
    can_undo: bool,
    can_redo: bool,
}

impl ToolbarViewModel {
    pub fn new(can_undo: bool, can_redo: bool) -> Self {
        Self { can_undo, can_redo }
    }
}

pub fn show(ui: &mut Ui, vm: &ToolbarViewModel, action_queue: &mut ActionRequestQueue) {
    ui.add_space(ui.spacing().item_spacing.y);
    ui.horizontal(|ui| {
        if button(ui, icon::ARROW_UNDO, "Undo", vm.can_undo).clicked() {
            action_queue.request(Action::Undo);
        }

        if button(ui, icon::ARROW_REDO, "Redo", vm.can_redo).clicked() {
            action_queue.request(Action::Redo);
        }

        ui.separator();

        if button(ui, icon::PLUS, "New Game", true).clicked() {
            action_queue.request(Action::RequestNewGameConfirm);
        }
    });
    ui.add_space(ui.spacing().item_spacing.y);
}

fn button(ui: &mut Ui, label: &str, hover_text: &str, enabled: bool) -> Response {
    let text_size = 60.0;
    ui.add_enabled(enabled, Button::new(RichText::new(label).size(text_size)))
        .on_hover_text(hover_text)
}
