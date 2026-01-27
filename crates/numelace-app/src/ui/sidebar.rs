use eframe::egui::{Button, CollapsingHeader, RichText, ScrollArea, Ui};

use crate::{
    app::GameStatus,
    state::{AppearanceSettings, HighlightSettings, Theme},
    ui::Action,
};

#[derive(Debug, Clone)]
pub struct SidebarViewModel<'a> {
    status: GameStatus,
    highlight_settings: &'a HighlightSettings,
    appearance_settings: &'a AppearanceSettings,
}

impl<'a> SidebarViewModel<'a> {
    pub fn new(
        status: GameStatus,
        highlight_settings: &'a HighlightSettings,
        appearance_settings: &'a AppearanceSettings,
    ) -> Self {
        Self {
            status,
            highlight_settings,
            appearance_settings,
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
            ui.heading("Settings");
            ui.indent("sidebar_settings", |ui| {
                let mut settings = vm.highlight_settings.clone();
                let mut changed = false;
                CollapsingHeader::new("Highlight")
                    .default_open(true)
                    .show(ui, |ui| {
                        changed |= ui
                            .checkbox(&mut settings.same_digit, "Same digit cells/notes")
                            .changed();
                        ui.label(RichText::new("Row/Col/Box Highlight"));
                        ui.indent("house_highlight", |ui| {
                            changed |= ui
                                .checkbox(&mut settings.house_selected, "Selected cell")
                                .changed();
                            changed |= ui
                                .checkbox(&mut settings.house_same_digit, "Same digit cells")
                                .changed();
                        });
                        if changed {
                            actions.push(Action::UpdateHighlightSettings(settings));
                        }
                    });

                let mut settings = vm.appearance_settings.clone();
                let mut changed = false;
                CollapsingHeader::new("Appearance")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Theme"));
                        ui.indent("theme", |ui| {
                            changed |= ui
                                .radio_value(&mut settings.theme, Theme::Light, "Light")
                                .changed();
                            changed |= ui
                                .radio_value(&mut settings.theme, Theme::Dark, "Dark")
                                .changed();
                        });
                    });
                if changed {
                    actions.push(Action::UpdateAppearanceSettings(settings));
                }
            });
        });
    });
    actions
}
