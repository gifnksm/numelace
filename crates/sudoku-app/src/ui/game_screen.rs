use eframe::egui::Ui;
use egui_extras::{Size, StripBuilder};
use sudoku_core::Position;
use sudoku_game::Game;

use crate::{
    app::GameStatus,
    ui::{self, Action},
};

pub fn show(
    ui: &mut Ui,
    game: &Game,
    status: GameStatus,
    selected_cell: Option<Position>,
) -> Vec<Action> {
    let mut actions = vec![];
    StripBuilder::new(ui)
        .size(Size::relative(0.75))
        .size(Size::relative(0.25))
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                StripBuilder::new(ui)
                    .size(Size::relative(9.0 / (9.0 + 2.0)))
                    .size(Size::relative(2.0 / (9.0 + 2.0)))
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            actions.extend(&super::grid::show(ui, game, selected_cell));
                        });
                        strip.cell(|ui| {
                            actions.extend(&super::keypad::show(ui, game, selected_cell));
                        });
                    });
            });
            strip.cell(|ui| {
                actions.extend(&ui::sidebar::show(ui, status));
            });
        });
    actions
}
