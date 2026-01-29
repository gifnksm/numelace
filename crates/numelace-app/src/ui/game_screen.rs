use eframe::egui::{self, Ui};
use egui_extras::{Size, StripBuilder};

use super::{grid, keypad};
use crate::{
    action::ActionRequestQueue,
    ui::{grid::GridViewModel, keypad::KeypadViewModel},
};

#[derive(Debug, Clone)]
pub struct GameScreenViewModel {
    pub grid_vm: GridViewModel,
    pub keypad_vm: KeypadViewModel,
}

impl GameScreenViewModel {
    pub fn new(grid_vm: GridViewModel, keypad_vm: KeypadViewModel) -> Self {
        Self { grid_vm, keypad_vm }
    }
}

pub fn show(ui: &mut Ui, vm: &GameScreenViewModel, action_queue: &mut ActionRequestQueue) {
    let grid_rows = 9.0;
    let keypad_rows = 2.0;
    let total_rows = grid_rows + keypad_rows;

    let grid_ratio = egui::vec2(1.0, grid_rows / total_rows);
    let spacing = ui.spacing().item_spacing;
    let spaces = spacing * egui::vec2(2.0, 3.0);
    let grid_size = ((ui.available_size() - spaces) * grid_ratio).min_elem();
    let keypad_size = grid_size / grid_rows * keypad_rows;

    StripBuilder::new(ui)
        .size(Size::remainder())
        .size(Size::exact(grid_size))
        .size(Size::remainder())
        .horizontal(|mut strip| {
            strip.empty();
            strip.cell(|ui| {
                StripBuilder::new(ui)
                    .size(Size::remainder())
                    .size(Size::exact(grid_size))
                    .size(Size::exact(spacing.y))
                    .size(Size::exact(keypad_size))
                    .size(Size::remainder())
                    .vertical(|mut strip| {
                        strip.empty();
                        strip.cell(|ui| {
                            grid::show(ui, &vm.grid_vm, action_queue);
                        });
                        strip.cell(|_ui| {}); // Spacer
                        strip.cell(|ui| {
                            keypad::show(ui, &vm.keypad_vm, action_queue);
                        });
                        strip.empty();
                    });
            });
            strip.empty();
        });
}
