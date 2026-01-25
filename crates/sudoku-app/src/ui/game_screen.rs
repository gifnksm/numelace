use eframe::egui::Ui;
use egui_extras::{Size, StripBuilder};

use crate::{
    app::GameStatus,
    ui::{self, Action, grid::GridViewModel, keypad::KeypadViewModel},
};

#[derive(Debug, Clone)]
pub struct GameScreenViewModel<'a> {
    grid_vm: GridViewModel<'a>,
    keypad_vm: KeypadViewModel,
    status: GameStatus,
}

impl<'a> GameScreenViewModel<'a> {
    pub fn new(grid_vm: GridViewModel<'a>, keypad_vm: KeypadViewModel, status: GameStatus) -> Self {
        Self {
            grid_vm,
            keypad_vm,
            status,
        }
    }
}

pub fn show(ui: &mut Ui, vm: &GameScreenViewModel<'_>) -> Vec<Action> {
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
                            actions.extend(&super::grid::show(ui, &vm.grid_vm));
                        });
                        strip.cell(|ui| {
                            actions.extend(&super::keypad::show(ui, &vm.keypad_vm));
                        });
                    });
            });
            strip.cell(|ui| {
                actions.extend(&ui::sidebar::show(ui, vm.status));
            });
        });
    actions
}
