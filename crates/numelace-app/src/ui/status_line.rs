use eframe::egui::{Align, Label, RichText, Ui, Vec2, Widget as _};

use crate::{
    state::{HintStage, HintState},
    ui::{
        icon,
        layout::{ComponentUnits, LayoutScale},
    },
};

#[derive(Debug, Clone)]
pub(crate) enum GameStatus<'a> {
    InProgress,
    Solved,
    Hint(&'a HintState),
}

#[derive(Debug, Clone)]
pub(crate) struct StatusLineViewModel<'a> {
    status: GameStatus<'a>,
}

impl<'a> StatusLineViewModel<'a> {
    #[must_use]
    pub(crate) fn new(status: GameStatus<'a>) -> Self {
        Self { status }
    }
}

#[must_use]
pub(crate) fn required_units() -> ComponentUnits {
    ComponentUnits::new(0.0, 0.5)
}

pub(crate) fn show(ui: &mut Ui, vm: &StatusLineViewModel, scale: &LayoutScale) {
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
            GameStatus::Hint(hint) => (
                match hint.stage {
                    HintStage::Stage1 => {
                        format!("{} Hint: Focus on the highlighted area.", icon::LIGHTBULB)
                    }
                    HintStage::Stage2 => {
                        format!("{} Hint: {}", icon::LIGHTBULB, hint.step.technique_name())
                    }
                    HintStage::Stage3 => {
                        format!("{} Hint: This step will make progress.", icon::LIGHTBULB)
                    }
                },
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
