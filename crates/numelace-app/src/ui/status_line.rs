use eframe::egui::{Align, Label, RichText, Ui, Vec2, Widget as _};

use crate::ui::{
    icon,
    layout::{ComponentUnits, LayoutScale},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Solved,
}

#[derive(Debug, Clone)]
pub struct StatusLineViewModel {
    status: GameStatus,
}

impl StatusLineViewModel {
    #[must_use]
    pub fn new(status: GameStatus) -> Self {
        Self { status }
    }
}

#[must_use]
pub fn required_units() -> ComponentUnits {
    ComponentUnits::new(0.0, 0.5)
}

pub fn show(ui: &mut Ui, vm: &StatusLineViewModel, scale: &LayoutScale) {
    let cell_size = scale.cell_size;
    ui.spacing_mut().item_spacing = Vec2::new(scale.spacing.x, 0.0);
    ui.horizontal(|ui| {
        let (status_text, status_color) = match vm.status {
            GameStatus::InProgress => (
                format!("{} Game in progress...", icon::HOURGLASS),
                ui.visuals().text_color(),
            ),
            GameStatus::Solved => (
                format!("{} Solved! Congratulations!", icon::TROPHY),
                ui.visuals().warn_fg_color,
            ),
        };
        Label::new(
            RichText::new(status_text)
                .color(status_color)
                .size(cell_size * 0.4),
        )
        .halign(Align::Max)
        .ui(ui);
    });
}
