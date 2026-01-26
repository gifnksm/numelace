use eframe::egui::{Button, CollapsingHeader, RichText, ScrollArea, Ui};

use crate::{
    app::{GameStatus, HighlightConfig, Theme, ThemeConfig},
    ui::Action,
};

#[derive(Debug, Clone)]
pub struct SidebarViewModel<'a> {
    status: GameStatus,
    highlight_config: &'a HighlightConfig,
    theme_config: &'a ThemeConfig,
}

impl<'a> SidebarViewModel<'a> {
    pub fn new(
        status: GameStatus,
        highlight_config: &'a HighlightConfig,
        theme_config: &'a ThemeConfig,
    ) -> Self {
        Self {
            status,
            highlight_config,
            theme_config,
        }
    }
}

pub fn show(ui: &mut Ui, vm: &SidebarViewModel) -> Vec<Action> {
    let mut actions = vec![];
    ui.vertical(|ui| {
        ui.group(|ui| {
            let status_text = match vm.status {
                GameStatus::InProgress => "Game in progress",
                GameStatus::Solved => "Congratulations! You solved the puzzle!",
            };
            let status_label = match vm.status {
                GameStatus::InProgress => RichText::new(status_text),
                GameStatus::Solved => RichText::new(status_text).color(ui.visuals().warn_fg_color),
            };
            ui.label(status_label.size(20.0));
            ui.add_space(8.0);
            let button = ui.add_sized(
                [ui.available_width(), 36.0],
                Button::new(RichText::new("New Game").size(20.0)),
            );
            if button.clicked() {
                actions.push(Action::RequestNewGameConfirm);
            }
        });

        ScrollArea::vertical().show(ui, |ui| {
            let mut hlc = vm.highlight_config.clone();
            let mut hlc_changed = false;
            CollapsingHeader::new("Highlight settings")
                .default_open(true)
                .show(ui, |ui| {
                    hlc_changed |= ui
                        .checkbox(&mut hlc.same_digit, "Same digit cells")
                        .changed();
                    ui.label(RichText::new("Row/Col/Box Highlight"));
                    ui.indent("rcb_highlight", |ui| {
                        hlc_changed |= ui
                            .checkbox(&mut hlc.rcb_selected, "Selected cell")
                            .changed();
                        hlc_changed |= ui
                            .checkbox(&mut hlc.rcb_same_digit, "Same digit cells")
                            .changed();
                    });
                    if hlc_changed {
                        actions.push(Action::UpdateHighlightConfig(hlc));
                    }
                });

            let mut theme_config = vm.theme_config.clone();
            let mut theme_changed = false;
            CollapsingHeader::new("Appearance settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(RichText::new("Theme"));
                    ui.indent("theme", |ui| {
                        theme_changed |= ui
                            .radio_value(&mut theme_config.theme, Theme::Light, "Light")
                            .changed();
                        theme_changed |= ui
                            .radio_value(&mut theme_config.theme, Theme::Dark, "Dark")
                            .changed();
                    });
                });
            if theme_changed {
                actions.push(Action::UpdateThemeConfig(theme_config));
            }
        });
    });
    actions
}
