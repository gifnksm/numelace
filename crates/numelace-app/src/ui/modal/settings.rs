use eframe::egui::{CollapsingHeader, Context, Id, Modal, ScrollArea, Sides, widgets};

use crate::{
    action::{ActionRequestQueue, SettingsAction, UiAction},
    state::{AssistSettings, HighlightSettings, NotesSettings, Settings},
    ui::icon,
};

#[derive(Debug, Clone)]
pub(crate) struct SettingsViewModel<'a> {
    settings: &'a Settings,
}

impl<'a> SettingsViewModel<'a> {
    #[must_use]
    pub(crate) fn new(settings: &'a Settings) -> Self {
        Self { settings }
    }
}

pub(crate) fn show(ctx: &Context, vm: &SettingsViewModel, action_queue: &mut ActionRequestQueue) {
    let modal = Modal::new(Id::new("settings_modal")).show(ctx, |ui| {
        ui.heading("Settings");
        let mut changed = false;
        let mut settings = vm.settings.clone();
        let Settings { assist } = &mut settings;
        ScrollArea::vertical().show(ui, |ui| {
            let AssistSettings {
                block_rule_violations,
                highlight,
                notes,
            } = assist;
            CollapsingHeader::new(format!("{} Assist", icon::BOLT))
                .default_open(true)
                .show(ui, |ui| {
                    changed |= ui
                        .checkbox(block_rule_violations, "Block rule violations")
                        .changed();

                    ui.label(format!("{} Highlight", icon::BRIGHTNESS));
                    ui.indent("highlight", |ui| {
                        let HighlightSettings {
                            same_digit,
                            house_selected,
                            house_same_digit,
                            conflict,
                        } = highlight;
                        changed |= ui.checkbox(same_digit, "Same digit cells/notes").changed();
                        changed |= ui
                            .checkbox(house_selected, "Selected cell's row/col/box")
                            .changed();
                        changed |= ui
                            .checkbox(house_same_digit, "Same digit cells' row/col/box")
                            .changed();
                        changed |= ui.checkbox(conflict, "Conflicting cells/notes").changed();
                    });

                    ui.label(format!("{} Notes", icon::PENCIL));
                    ui.indent("notes", |ui| {
                        let NotesSettings {
                            auto_remove_peer_notes_on_fill,
                            auto_fill_notes_on_new_or_reset,
                        } = notes;
                        changed |= ui
                            .checkbox(
                                auto_remove_peer_notes_on_fill,
                                "Auto-remove row/col/box notes on fill",
                            )
                            .changed();
                        changed |= ui
                            .checkbox(
                                auto_fill_notes_on_new_or_reset,
                                "Auto-fill notes on new game/reset",
                            )
                            .changed();
                    });
                });

            CollapsingHeader::new(format!("{} Appearance", icon::PALETTE))
                .default_open(true)
                .show(ui, |ui| {
                    widgets::global_theme_preference_buttons(ui);
                });
        });

        Sides::new().show(
            ui,
            |_ui| {},
            |ui| {
                if ui.button(format!("{} Close", icon::CHECK)).clicked() {
                    ui.close();
                }
            },
        );
        if changed {
            action_queue.request(SettingsAction::UpdateSettings(settings).into());
        }
    });
    if modal.should_close() {
        action_queue.request(UiAction::CloseModal.into());
    }
}
