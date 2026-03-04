use eframe::egui::Context;

pub(crate) use self::{new_game_options::NewGameOptionsViewModel, settings::SettingsViewModel};
use crate::action::{ActionRequestQueue, ModalRequest};

mod dialogs;
mod new_game_options;
mod settings;

pub(crate) fn show(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    modal_request: &mut ModalRequest,
    new_game_options_vm: &NewGameOptionsViewModel,
    settings_vm: &SettingsViewModel,
) {
    match modal_request {
        ModalRequest::Confirm { kind, responder } => {
            dialogs::show_confirm(ctx, *kind, responder);
        }
        ModalRequest::Alert { kind, responder } => {
            dialogs::show_alert(ctx, kind, responder);
        }
        ModalRequest::NewGameOptions {
            can_cancel,
            responder,
        } => {
            new_game_options::show(
                ctx,
                new_game_options_vm,
                action_queue,
                *can_cancel,
                responder,
            );
        }
        ModalRequest::Settings => {
            settings::show(ctx, settings_vm, action_queue);
        }
    }
}
