use eframe::egui::{Button, Context, Id, Modal, Response, RichText, Sides, Ui};

use crate::{
    action::{
        ActionRequestQueue, ConfirmKind, ConfirmResponder, ConfirmResult, Responder,
        SolvabilityDialogResult, SolvabilityResponder, UiAction,
    },
    state::SolvabilityState,
    ui::icon,
};

struct DialogResult {
    should_close: bool,
}

fn show_dialog<Heading, Body, Buttons>(
    ctx: &Context,
    id: Id,
    heading: Heading,
    body: Body,
    buttons: Buttons,
) -> DialogResult
where
    Heading: Into<RichText>,
    Body: FnOnce(&mut Ui),
    Buttons: FnOnce(&mut Ui),
{
    let modal = Modal::new(id).show(ctx, |ui| {
        ui.heading(heading);
        ui.add_space(4.0);

        body(ui);
        ui.add_space(8.0);

        Sides::new().show(ui, |_ui| {}, buttons);
    });

    DialogResult {
        should_close: modal.should_close(),
    }
}

fn request_focus_if_none(ui: &Ui, response: &Response) {
    if ui.memory(|memory| memory.focused().is_none()) {
        response.request_focus();
    }
}

fn primary_button(ui: &mut Ui, label: String, request_focus: bool) -> Response {
    let response = ui.button(label);
    if request_focus {
        request_focus_if_none(ui, &response);
    }
    response
}

fn disabled_button(ui: &mut Ui, label: String) {
    ui.add_enabled(false, Button::new(label));
}

fn send_response<T>(responder: &mut Option<Responder<T>>, response: T) {
    if let Some(responder) = responder.take() {
        let _ = responder.send(response);
    }
}

impl ConfirmKind {
    fn id(self) -> Id {
        match self {
            ConfirmKind::NewGame => Id::new("new_game_confirm"),
            ConfirmKind::ResetInputs => Id::new("reset_inputs_confirm"),
        }
    }

    fn heading(self) -> &'static str {
        match self {
            ConfirmKind::NewGame => "New Game?",
            ConfirmKind::ResetInputs => "Reset Inputs?",
        }
    }

    fn label(self) -> &'static str {
        match self {
            ConfirmKind::NewGame => "Start a new game? Current progress will be lost.",
            ConfirmKind::ResetInputs => {
                "Clear all your inputs and return to the initial puzzle state?"
            }
        }
    }

    fn confirm_label(self) -> &'static str {
        match self {
            ConfirmKind::NewGame => "New Game",
            ConfirmKind::ResetInputs => "Reset Inputs",
        }
    }
}

pub(crate) fn show_confirm(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    kind: ConfirmKind,
    responder: &mut Option<ConfirmResponder>,
) {
    let DialogResult { should_close } = show_dialog(
        ctx,
        kind.id(),
        kind.heading(),
        |ui: &mut Ui| {
            ui.label(kind.label());
        },
        |ui: &mut Ui| {
            let confirm = primary_button(
                ui,
                format!("{} {}", icon::CHECK, kind.confirm_label()),
                true,
            );
            if confirm.clicked() {
                send_response(responder, ConfirmResult::Confirmed);
                ui.close();
            }

            let cancel = ui.button(format!("{} Cancel", icon::CANCEL));
            if cancel.clicked() {
                send_response(responder, ConfirmResult::Cancelled);
                ui.close();
            }
        },
    );

    if should_close {
        send_response(responder, ConfirmResult::Cancelled);
        action_queue.request(UiAction::CloseModal.into());
    }
}

pub(crate) fn show_solvability(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    state: &SolvabilityState,
    responder: &mut Option<SolvabilityResponder>,
) {
    match state {
        SolvabilityState::Inconsistent => {
            show_solvability_inconsistent(ctx, action_queue, responder);
        }
        SolvabilityState::NoSolution => show_solvability_no_solution(ctx, action_queue, responder),
        SolvabilityState::Solvable {
            with_user_notes: true,
            stats: _stats,
        } => show_solvability_solvable(ctx, action_queue, responder),
        SolvabilityState::Solvable {
            with_user_notes: false,
            stats: _stats,
        } => show_solvability_notes_maybe_incorrect(ctx, action_queue, responder),
    }
}

fn show_solvability_inconsistent(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    responder: &mut Option<SolvabilityResponder>,
) {
    let DialogResult { should_close } = show_dialog(
        ctx,
        Id::new("solvability_result"),
        "Board Inconsistent",
        |ui: &mut Ui| {
            ui.label("A conflict or a no-candidate cell was detected. We recommend undoing to the last consistent state.");
        },
        |ui: &mut Ui| {
            disabled_button(ui, format!("{} Undo (coming soon)", icon::ARROW_UNDO));
            if ui.button(format!("{} Cancel", icon::CANCEL)).clicked() {
                send_response(responder, SolvabilityDialogResult::Close);
                ui.close();
            }
        },
    );

    if should_close {
        send_response(responder, SolvabilityDialogResult::Close);
        action_queue.request(UiAction::CloseModal.into());
    }
}

fn show_solvability_no_solution(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    responder: &mut Option<SolvabilityResponder>,
) {
    let DialogResult { should_close } = show_dialog(
        ctx,
        Id::new("solvability_result"),
        "No Solution Found",
        |ui: &mut Ui| {
            ui.label("No solution exists from the current state. We recommend undoing to the last solvable state.");
        },
        |ui: &mut Ui| {
            disabled_button(ui, format!("{} Undo (coming soon)", icon::ARROW_UNDO));
            if ui.button(format!("{} Cancel", icon::CANCEL)).clicked() {
                send_response(responder, SolvabilityDialogResult::Close);
                ui.close();
            }
        },
    );

    if should_close {
        send_response(responder, SolvabilityDialogResult::Close);
        action_queue.request(UiAction::CloseModal.into());
    }
}

fn show_solvability_solvable(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    responder: &mut Option<SolvabilityResponder>,
) {
    let DialogResult { should_close } = show_dialog(
        ctx,
        Id::new("solvability_result"),
        "Solvable",
        |ui: &mut Ui| {
            ui.label("A solution is still possible from the current state.");
        },
        |ui: &mut Ui| {
            let ok = primary_button(ui, format!("{} OK", icon::CHECK), true);
            if ok.clicked() {
                send_response(responder, SolvabilityDialogResult::Close);
                ui.close();
            }
        },
    );

    if should_close {
        send_response(responder, SolvabilityDialogResult::Close);
        action_queue.request(UiAction::CloseModal.into());
    }
}

fn show_solvability_notes_maybe_incorrect(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    responder: &mut Option<SolvabilityResponder>,
) {
    let DialogResult { should_close } = show_dialog(
        ctx,
        Id::new("solvability_result"),
        "Notes May Be Incorrect",
        |ui: &mut Ui| {
            ui.label("A solution exists when ignoring notes. Rebuild candidates now?");
        },
        |ui: &mut Ui| {
            let rebuild = primary_button(ui, format!("{} Rebuild", icon::CHECK), true);
            if rebuild.clicked() {
                send_response(responder, SolvabilityDialogResult::RebuildNotes);
                ui.close();
            }

            if ui.button(format!("{} Cancel", icon::CANCEL)).clicked() {
                send_response(responder, SolvabilityDialogResult::Close);
                ui.close();
            }
        },
    );

    if should_close {
        send_response(responder, SolvabilityDialogResult::Close);
        action_queue.request(UiAction::CloseModal.into());
    }
}
