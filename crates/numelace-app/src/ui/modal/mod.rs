use eframe::egui::Context;

pub(crate) use self::settings::SettingsViewModel;
use crate::action::{ActionRequestQueue, ModalRequest};

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
        ModalRequest::Alert { kind, responder } => {
            dialogs::show_alert(ctx, action_queue, *kind, responder);
        }
        ModalRequest::Settings => {
            settings::show(ctx, settings_vm, action_queue);
        }
    }
}
