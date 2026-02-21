use std::borrow::Cow;

use eframe::egui::{CollapsingHeader, Context, Id, Modal, Response, RichText, Sides, Ui};

use crate::{
    action::{
        ActionRequestQueue, AlertKind, AlertResponder, AlertResult, ConfirmKind, ConfirmResponder,
        ConfirmResult, Responder, UiAction,
    },
    ui::icon,
    worker::tasks::SolvabilityStatsDto,
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

fn send_response<T>(responder: &mut Option<Responder<T>>, response: T) {
    if let Some(responder) = responder.take() {
        let _ = responder.send(response);
    }
}

struct ConfirmDialogSpec {
    id: Id,
    heading: &'static str,
    label: &'static str,
    confirm_label: &'static str,
    confirm_icon: &'static str,
}

impl ConfirmKind {
    fn spec(self) -> ConfirmDialogSpec {
        match self {
            ConfirmKind::NewGame => ConfirmDialogSpec {
                id: Id::new("new_game_confirm"),
                heading: "New Game?",
                label: "Start a new game? Current progress will be lost.",
                confirm_label: "New Game",
                confirm_icon: icon::CHECK,
            },
            ConfirmKind::ResetInputs => ConfirmDialogSpec {
                id: Id::new("reset_inputs_confirm"),
                heading: "Reset Inputs?",
                label: "Clear all your inputs and return to the initial puzzle state?",
                confirm_label: "Reset Inputs",
                confirm_icon: icon::CHECK,
            },
            ConfirmKind::SolvabilityInconsistent => ConfirmDialogSpec {
                id: Id::new("solvability_result"),
                heading: "Board Inconsistent",
                label: "A conflict or a no-candidate cell was detected. We recommend undoing to the last consistent state.",
                confirm_label: "Undo",
                confirm_icon: icon::ARROW_UNDO,
            },
            ConfirmKind::SolvabilityNoSolution => ConfirmDialogSpec {
                id: Id::new("solvability_result"),
                heading: "No Solution Found",
                label: "No solution exists from the current state. We recommend undoing to the last solvable state.",
                confirm_label: "Undo",
                confirm_icon: icon::ARROW_UNDO,
            },
            ConfirmKind::SolvabilityNotesMaybeIncorrect => ConfirmDialogSpec {
                id: Id::new("solvability_result"),
                heading: "Notes May Be Incorrect",
                label: "A solution exists when ignoring notes. Rebuild candidates now?",
                confirm_label: "Rebuild",
                confirm_icon: icon::CHECK,
            },
            ConfirmKind::HintInconsistent => ConfirmDialogSpec {
                id: Id::new("hint_inconsistent"),
                heading: "Board Inconsistent",
                label: "A conflict or a no-candidate cell was detected. We recommend undoing to the last consistent state.",
                confirm_label: "Undo",
                confirm_icon: icon::ARROW_UNDO,
            },
            ConfirmKind::HintNotesMaybeIncorrect => ConfirmDialogSpec {
                id: Id::new("hint_notes_maybe_incorrect"),
                heading: "Notes May Be Incorrect",
                label: "A hint exists when ignoring notes. Rebuild candidates now?",
                confirm_label: "Rebuild",
                confirm_icon: icon::CHECK,
            },
        }
    }
}

#[derive(Debug)]
enum AlertBody<'a> {
    Text(Cow<'a, str>),
    SolvabilityStats {
        summary: Cow<'a, str>,
        stats: &'a SolvabilityStatsDto,
    },
}

struct AlertDialogSpec<'a> {
    id: Id,
    heading: &'static str,
    body: AlertBody<'a>,
    ok_label: &'static str,
}

impl AlertKind {
    fn spec(&self) -> AlertDialogSpec<'_> {
        match self {
            AlertKind::SolvabilitySolvable { stats } => AlertDialogSpec {
                id: Id::new("solvability_result"),
                heading: "Solvable",
                body: AlertBody::SolvabilityStats {
                    summary: Cow::Borrowed("A solution is still possible from the current state."),
                    stats,
                },
                ok_label: "OK",
            },
            AlertKind::SolvabilityUndoNotice { steps } => AlertDialogSpec {
                id: Id::new("solvability_undo_notice"),
                heading: "Undo Complete",
                body: AlertBody::Text(Cow::Owned(format!(
                    "Undid {steps} step(s) to return to a solvable state."
                ))),
                ok_label: "OK",
            },
            AlertKind::SolvabilityUndoNotFound => AlertDialogSpec {
                id: Id::new("solvability_undo_not_found"),
                heading: "No Solution Found",
                body: AlertBody::Text(Cow::Borrowed("Undo did not find a solvable state.")),
                ok_label: "OK",
            },
            AlertKind::HintUndoNotice { steps } => AlertDialogSpec {
                id: Id::new("hint_undo_notice"),
                heading: "Undo Complete",
                body: AlertBody::Text(Cow::Owned(format!(
                    "Undid {steps} step(s) to return to a consistent state."
                ))),
                ok_label: "OK",
            },
            AlertKind::HintStuckNoStep => AlertDialogSpec {
                id: Id::new("hint_stuck_no_step"),
                heading: "No Hint Found",
                body: AlertBody::Text(Cow::Borrowed(
                    "No applicable techniques found from the current state.",
                )),
                ok_label: "OK",
            },
            AlertKind::HintStuckAfterRollback => AlertDialogSpec {
                id: Id::new("hint_stuck_after_rollback"),
                heading: "No Hint Found",
                body: AlertBody::Text(Cow::Borrowed(
                    "The conflict was resolved, but no next step is available.",
                )),
                ok_label: "OK",
            },
            AlertKind::HintInconsistentAfterRollback => AlertDialogSpec {
                id: Id::new("hint_inconsistent_after_rollback"),
                heading: "No Hint Found",
                body: AlertBody::Text(Cow::Borrowed("Undo did not find a consistent state.")),
                ok_label: "OK",
            },
        }
    }
}

pub(crate) fn show_confirm(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    kind: ConfirmKind,
    responder: &mut Option<ConfirmResponder>,
) {
    let spec = kind.spec();
    let DialogResult { should_close } = show_dialog(
        ctx,
        spec.id,
        spec.heading,
        |ui: &mut Ui| {
            ui.label(spec.label);
        },
        |ui: &mut Ui| {
            let confirm = primary_button(
                ui,
                format!("{} {}", spec.confirm_icon, spec.confirm_label),
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

pub(crate) fn show_alert(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    kind: &AlertKind,
    responder: &mut Option<AlertResponder>,
) {
    let spec = kind.spec();
    let DialogResult { should_close } = show_dialog(
        ctx,
        spec.id,
        spec.heading,
        |ui: &mut Ui| match &spec.body {
            AlertBody::Text(text) => {
                ui.label(text.as_ref());
            }
            AlertBody::SolvabilityStats { summary, stats } => {
                ui.label(summary.as_ref());
                CollapsingHeader::new("Solving details")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label(format!("Total steps: {}", stats.total_steps));
                        let mut shown = false;
                        for entry in stats
                            .technique_counts
                            .iter()
                            .filter(|entry| entry.count > 0)
                        {
                            shown = true;
                            ui.label(format!("{}: {}", entry.name, entry.count));
                        }
                        if !shown {
                            ui.label("No techniques applied.");
                        }
                    });
            }
        },
        |ui: &mut Ui| {
            let ok = primary_button(ui, format!("{} {}", icon::CHECK, spec.ok_label), true);
            if ok.clicked() {
                send_response(responder, AlertResult::Ok);
                ui.close();
            }
        },
    );

    if should_close {
        send_response(responder, AlertResult::Ok);
        action_queue.request(UiAction::CloseModal.into());
    }
}
