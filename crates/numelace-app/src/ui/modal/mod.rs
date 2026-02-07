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
        ModalRequest::Confirm { kind, responder } => {
            dialogs::show_confirm(ctx, action_queue, *kind, responder);
        }
        ModalRequest::Settings => {
            settings::show(ctx, settings_vm, action_queue);
        }
        ModalRequest::CheckSolvabilityResult { state, responder } => {
            dialogs::show_solvability(ctx, action_queue, state, responder);
        }
        ModalRequest::SolvabilityUndoNotice { steps, responder } => {
            dialogs::show_solvability_undo_notice(ctx, action_queue, *steps, responder);
        }
        ModalRequest::SolvabilityUndoNotFound => {
            dialogs::show_solvability_undo_not_found(ctx, action_queue);
        }
    }
}
