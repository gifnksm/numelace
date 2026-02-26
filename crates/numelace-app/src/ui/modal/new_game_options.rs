use eframe::egui::{
    Checkbox, CollapsingHeader, Context, DragValue, Id, Modal, Response, Sides, TextEdit, Ui,
};
use numelace_solver::technique;

use crate::{
    action::{ActionRequestQueue, NewGameOptionsResponder, UiAction, UpdateStateAction},
    state::{DifficultyPreset, NewGameOptions},
    ui::icon,
};

#[derive(Debug, Clone)]
pub(crate) struct NewGameOptionsViewModel<'a> {
    new_game_options: &'a NewGameOptions,
}

impl<'a> NewGameOptionsViewModel<'a> {
    #[must_use]
    pub(crate) fn new(settings: &'a NewGameOptions) -> Self {
        Self {
            new_game_options: settings,
        }
    }
}

pub(crate) fn show(
    ctx: &Context,
    vm: &NewGameOptionsViewModel,
    action_queue: &mut ActionRequestQueue,
    can_cancel: bool,
    responder: &mut Option<NewGameOptionsResponder>,
) {
    let mut draft = vm.new_game_options.clone();
    let modal = Modal::new(Id::new("new_game_options_modal")).show(ctx, |ui| {
        ui.heading("New Game");
        ui.label("Choose difficulty and techniques to generate a new puzzle.");

        let mut changed = false;

        ui.separator();
        ui.label("Difficulty");
        for preset in DifficultyPreset::all() {
            let response = ui.radio_value(&mut draft.difficulty, preset, preset.label());
            if response.clicked() {
                changed = true;
                draft.apply_preset(preset);
            }
        }

        CollapsingHeader::new("Techniques")
            .default_open(false)
            .show(ui, |ui| {
                for technique in technique::all_techniques() {
                    let mut enabled = draft.is_technique_enabled(technique.id());
                    let can_toggle = !technique.tier().is_fundamental();
                    if ui
                        .add_enabled(can_toggle, Checkbox::new(&mut enabled, technique.name()))
                        .changed()
                    {
                        changed = true;
                        draft.set_technique_enabled(technique.id(), enabled);
                    }
                }
            });

        ui.separator();
        ui.label("Seed (optional)");
        changed |= ui
            .add(TextEdit::singleline(&mut draft.seed).hint_text("Leave blank for random"))
            .changed();

        ui.separator();
        ui.label("Generation attempts");
        changed |= ui
            .add(
                DragValue::new(&mut draft.max_attempts)
                    .speed(1)
                    .range(1..=10000),
            )
            .changed();

        Sides::new().show(
            ui,
            |_ui| {},
            |ui| {
                let response = ui.button(format!("{} Generate", icon::CHECK));
                request_focus_if_none(ui, &response);
                if response.clicked() {
                    let mut response = draft.clone();
                    response.seed = response.seed.trim().to_string();
                    send_response(responder, Some(response));
                    action_queue.request(UiAction::CloseModal.into());
                    ui.close();
                }
                if can_cancel && ui.button(format!("{} Cancel", icon::CANCEL)).clicked() {
                    send_response(responder, None);
                    action_queue.request(UiAction::CloseModal.into());
                    ui.close();
                }
            },
        );
        if changed {
            action_queue.request(UpdateStateAction::UpdateNewGameOptions(draft.clone()).into());
        }
    });
    if can_cancel && modal.should_close() {
        send_response(responder, None);
        action_queue.request(UiAction::CloseModal.into());
    }
}

fn request_focus_if_none(ui: &Ui, response: &Response) {
    if ui.memory(|memory| memory.focused().is_none()) {
        response.request_focus();
    }
}

fn send_response(
    responder: &mut Option<NewGameOptionsResponder>,
    response: Option<NewGameOptions>,
) {
    if let Some(responder) = responder.take() {
        let _ = responder.send(response);
    }
}
