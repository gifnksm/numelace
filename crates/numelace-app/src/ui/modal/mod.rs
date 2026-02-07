use eframe::egui::Context;

use crate::action::{ActionRequestQueue, ModalRequest};

pub(crate) use self::settings::SettingsViewModel;

mod dialogs;
mod settings;

pub(crate) fn show(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    modal_request: &mut ModalRequest,
    settings_vm: &SettingsViewModel,
) {
    match modal_request {
        ModalRequest::NewGameConfirm(responder) => {
            dialogs::show_new_game_confirm(ctx, action_queue, responder);
        }
        ModalRequest::ResetCurrentPuzzleConfirm(responder) => {
            dialogs::show_reset_current_puzzle_confirm(ctx, action_queue, responder);
        }
        ModalRequest::Settings => {
            settings::show(ctx, settings_vm, action_queue);
        }
        ModalRequest::CheckSolvabilityResult { state, responder } => {
            dialogs::show_solvability(ctx, action_queue, state, responder);
        }
    }
}
