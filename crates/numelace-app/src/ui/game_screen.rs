use eframe::{
    egui::{Ui, Vec2},
    emath::GUI_ROUNDING,
};
use egui_extras::{Size, StripBuilder};

use super::{grid, keypad, toolbar};
use crate::{
    action::ActionRequestQueue,
    ui::{
        grid::GridViewModel,
        keypad::KeypadViewModel,
        layout::LayoutScale,
        status_line::{self, StatusLineViewModel},
        toolbar::ToolbarViewModel,
    },
};

#[derive(Debug, Clone)]
pub struct GameScreenViewModel {
    pub toolbar: ToolbarViewModel,
    pub status_line: StatusLineViewModel,
    pub grid: GridViewModel,
    pub keypad: KeypadViewModel,
}

impl GameScreenViewModel {
    #[must_use]
    pub fn new(
        toolbar: ToolbarViewModel,
        status_line: StatusLineViewModel,
        grid: GridViewModel,
        keypad: KeypadViewModel,
    ) -> Self {
        Self {
            toolbar,
            status_line,
            grid,
            keypad,
        }
    }
}

pub fn show(ui: &mut Ui, vm: &GameScreenViewModel, action_queue: &mut ActionRequestQueue) {
    let avail = ui.available_size();
    let toolbar_units = toolbar::required_units();
    let grid_units = grid::required_units();
    let status_units = status_line::required_units();
    let keypad_units = keypad::required_units();

    let width_units = toolbar_units
        .width
        .max(grid_units.width)
        .max(status_units.width)
        .max(keypad_units.width);
    let height_units =
        toolbar_units.height + grid_units.height + status_units.height + keypad_units.height;
    let cell_size_width = avail.x / width_units;
    let cell_size_height = avail.y / height_units;
    let cell_size = round_cell_size(cell_size_width.min(cell_size_height).max(0.0));

    let scale = LayoutScale::new(cell_size);
    let toolbar_height = toolbar_units.height * cell_size;
    let status_height = status_units.height * cell_size;
    let grid_height = grid_units.height * cell_size;
    let grid_width = grid_height;
    let keypad_height = keypad_units.height * cell_size;

    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        with_horizontal_center(ui, grid_width, |ui| {
            StripBuilder::new(ui)
                .size(Size::exact(toolbar_height))
                .size(Size::remainder())
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        toolbar::show(ui, &vm.toolbar, &scale, action_queue);
                    });
                    strip.cell(|ui| {
                        StripBuilder::new(ui)
                            .size(Size::remainder())
                            .size(Size::exact(status_height))
                            .size(Size::exact(grid_height))
                            .size(Size::exact(keypad_height))
                            .size(Size::remainder())
                            .vertical(|mut strip| {
                                strip.empty();
                                strip.cell(|ui| {
                                    status_line::show(ui, &vm.status_line, &scale);
                                });
                                strip.cell(|ui| {
                                    grid::show(ui, &vm.grid, &scale, action_queue);
                                });
                                strip.cell(|ui| {
                                    keypad::show(ui, &vm.keypad, &scale, action_queue);
                                });
                                strip.empty();
                            });
                    });
                });
        });
    });
}

/// Snap the computed cell size to the GUI rounding grid so layout and rendering
/// agree on pixel boundaries and avoid cumulative rounding drift.
/// The `0.01` factor forces a 1/100-cell quantization first, then snaps that
/// to the GUI rounding unit. This guarantees that any value expressed as a
/// multiple of `cell_size * 0.01` remains representable after rounding.
fn round_cell_size(raw: f32) -> f32 {
    (raw * 0.01 / GUI_ROUNDING).floor() / 0.01 * GUI_ROUNDING
}

fn with_horizontal_center(ui: &mut Ui, width: f32, add: impl FnOnce(&mut Ui)) {
    StripBuilder::new(ui)
        .size(Size::remainder())
        .size(Size::exact(width))
        .size(Size::remainder())
        .horizontal(|mut strip| {
            strip.empty();
            strip.cell(add);
            strip.empty();
        });
}
