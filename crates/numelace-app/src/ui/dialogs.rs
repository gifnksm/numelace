use eframe::egui::{Id, Key, Modal, Sides, Ui};

use crate::ui::Action;

pub fn show_new_game_confirm(ui: &mut Ui, show: &mut bool) -> Vec<Action> {
    let mut actions = vec![];
    let modal = Modal::new(Id::new("new_game_confirm")).show(ui.ctx(), |ui| {
        ui.heading("New Game?");
        ui.add_space(4.0);
        ui.label("Start a new game? Current progress will be lost.");
        ui.add_space(8.0);

        Sides::new().show(
            ui,
            |_ui| {},
            |ui| {
                let new_game = ui.button("New Game");
                if ui.memory(|memory| memory.focused().is_none()) {
                    new_game.request_focus();
                }
                if new_game.clicked() {
                    actions.push(Action::NewGame);
                    ui.close();
                }
                if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                    ui.close();
                }
            },
        );
    });
    if modal.should_close() {
        *show = false;
    }
    actions
}
