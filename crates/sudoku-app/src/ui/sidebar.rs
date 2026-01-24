use eframe::egui::{RichText, Ui};

use crate::{app::GameStatus, ui::Action};

pub fn show(ui: &mut Ui, status: GameStatus) -> Vec<Action> {
    let mut actions = vec![];
    ui.vertical(|ui| {
        let text = match status {
            GameStatus::InProgress => "Game in progress",
            GameStatus::Solved => "Congratulations! You solved the puzzle!",
        };
        ui.label(RichText::new(text).size(20.0));
        if ui.button(RichText::new("New Game").size(20.0)).clicked() {
            actions.push(Action::NewGame);
        }
    });
    actions
}
