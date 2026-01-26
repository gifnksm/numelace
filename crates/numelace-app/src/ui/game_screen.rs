use eframe::egui::{self, Ui};
use egui_extras::{Size, StripBuilder};

use crate::ui::{
    self, Action, grid::GridViewModel, keypad::KeypadViewModel, sidebar::SidebarViewModel,
};

#[derive(Debug, Clone)]
#[expect(clippy::struct_field_names)]
pub struct GameScreenViewModel<'a> {
    grid_vm: GridViewModel<'a>,
    keypad_vm: KeypadViewModel,
    sidebar_vm: SidebarViewModel<'a>,
}

impl<'a> GameScreenViewModel<'a> {
    pub fn new(
        grid_vm: GridViewModel<'a>,
        keypad_vm: KeypadViewModel,
        sidebar_vm: SidebarViewModel<'a>,
    ) -> Self {
        Self {
            grid_vm,
            keypad_vm,
            sidebar_vm,
        }
    }
}

pub fn show(ui: &mut Ui, vm: &GameScreenViewModel<'_>) -> Vec<Action> {
    let mut actions = vec![];
    let grid_ratio = egui::vec2(0.75, 9.0 / (9.0 + 2.0));
    let spacing = ui.spacing().item_spacing;
    let adjusted_size = ((ui.available_size() - spacing) * grid_ratio).min_elem();
    StripBuilder::new(ui)
        .size(Size::exact(adjusted_size))
        .size(Size::exact(spacing.x))
        .size(Size::remainder())
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                StripBuilder::new(ui)
                    .size(Size::exact(adjusted_size))
                    .size(Size::exact(spacing.y))
                    .size(Size::remainder())
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            actions.extend(super::grid::show(ui, &vm.grid_vm));
                        });
                        strip.cell(|_ui| {}); // Spacer
                        strip.cell(|ui| {
                            actions.extend(super::keypad::show(ui, &vm.keypad_vm));
                        });
                    });
            });
            strip.cell(|_ui| {}); // Spacer
            strip.cell(|ui| {
                actions.extend(ui::sidebar::show(ui, &vm.sidebar_vm));
            });
        });
    actions
}
