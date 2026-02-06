use eframe::egui::{Button, Context, Id, Modal, Response, RichText, Sides, Ui};

use crate::{
    action::{Action, ActionRequestQueue, NotesFillScope},
    state::SolvabilityState,
    ui::icon,
};

fn show_dialog<Heading, Body, Buttons>(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    id: Id,
    heading: Heading,
    body: Body,
    buttons: Buttons,
) where
    Heading: Into<RichText>,
    Body: FnOnce(&mut Ui),
    Buttons: FnOnce(&mut Ui, &mut ActionRequestQueue),
{
    let modal = Modal::new(id).show(ctx, |ui| {
        ui.heading(heading);
        ui.add_space(4.0);

        body(ui);
        ui.add_space(8.0);

        Sides::new().show(
            ui,
            |_ui| {},
            |ui| {
                buttons(ui, action_queue);
            },
        );
    });

    if modal.should_close() {
        action_queue.request(Action::CloseModal);
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

fn close_button(ui: &mut Ui, label: String) {
    if ui.button(label).clicked() {
        ui.close();
    }
}

fn primary_close_button(ui: &mut Ui, label: String) {
    let response = primary_button(ui, label, true);
    if response.clicked() {
        ui.close();
    }
}

fn cancel_button(ui: &mut Ui) {
    close_button(ui, format!("{} Cancel", icon::CANCEL));
}

fn disabled_button(ui: &mut Ui, label: String) {
    ui.add_enabled(false, Button::new(label));
}

fn action_button(
    ui: &mut Ui,
    action_queue: &mut ActionRequestQueue,
    label: String,
    request_focus: bool,
    action: Action,
) {
    let response = primary_button(ui, label, request_focus);
    if response.clicked() {
        action_queue.request(action);
        ui.close();
    }
}

pub fn show_new_game_confirm(ctx: &Context, action_queue: &mut ActionRequestQueue) {
    show_dialog(
        ctx,
        action_queue,
        Id::new("new_game_confirm"),
        "New Game?",
        |ui: &mut Ui| {
            ui.label("Start a new game? Current progress will be lost.");
        },
        |ui: &mut Ui, action_queue| {
            action_button(
                ui,
                action_queue,
                format!("{} New Game", icon::CHECK),
                true,
                Action::StartNewGame,
            );
            cancel_button(ui);
        },
    );
}

pub fn show_reset_current_puzzle_confirm(ctx: &Context, action_queue: &mut ActionRequestQueue) {
    show_dialog(
        ctx,
        action_queue,
        Id::new("reset_current_puzzle_confirm"),
        "Reset Puzzle?",
        |ui: &mut Ui| {
            ui.label("Clear all your inputs and return to the initial puzzle state?");
        },
        |ui: &mut Ui, action_queue| {
            action_button(
                ui,
                action_queue,
                format!("{} Reset", icon::CHECK),
                true,
                Action::ResetCurrentPuzzle,
            );
            cancel_button(ui);
        },
    );
}

pub fn show_solvability(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    state: &SolvabilityState,
) {
    match state {
        SolvabilityState::Inconsistent => {
            show_dialog(
                ctx,
                action_queue,
                Id::new("solvability_result"),
                "Board Inconsistent",
                |ui: &mut Ui| {
                    ui.label("A conflict or a no-candidate cell was detected. We recommend undoing to the last consistent state.");
                },
                |ui: &mut Ui, _action_queue: &mut ActionRequestQueue| {
                    disabled_button(ui, format!("{} Undo (coming soon)", icon::ARROW_UNDO));
                    cancel_button(ui);
                },
            );
        }
        SolvabilityState::NoSolution => {
            show_dialog(
                ctx,
                action_queue,
                Id::new("solvability_result"),
                "No Solution Found",
                |ui: &mut Ui| {
                    ui.label("No solution exists from the current state. We recommend undoing to the last solvable state.");
                },
                |ui: &mut Ui, _action_queue: &mut ActionRequestQueue| {
                    disabled_button(ui, format!("{} Undo (coming soon)", icon::ARROW_UNDO));
                    cancel_button(ui);
                },
            );
        }
        SolvabilityState::Solvable {
            with_user_notes: true,
            stats: _stats,
        } => {
            show_dialog(
                ctx,
                action_queue,
                Id::new("solvability_result"),
                "Solvable",
                |ui: &mut Ui| {
                    ui.label("A solution is still possible from the current state.");
                },
                |ui: &mut Ui, _action_queue: &mut ActionRequestQueue| {
                    primary_close_button(ui, format!("{} OK", icon::CHECK));
                },
            );
        }
        SolvabilityState::Solvable {
            with_user_notes: false,
            stats: _stats,
        } => {
            show_dialog(
                ctx,
                action_queue,
                Id::new("solvability_result"),
                "Notes May Be Incorrect",
                |ui: &mut Ui| {
                    ui.label("A solution exists when ignoring notes. Rebuild candidates now?");
                },
                |ui: &mut Ui, action_queue: &mut ActionRequestQueue| {
                    action_button(
                        ui,
                        action_queue,
                        format!("{} Rebuild", icon::CHECK),
                        true,
                        Action::AutoFillNotes {
                            scope: NotesFillScope::AllCells,
                        },
                    );
                    cancel_button(ui);
                },
            );
        }
    }
}
