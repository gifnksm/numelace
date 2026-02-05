use eframe::egui::{Button, Context, Id, Modal, Sides};

use crate::{
    action::{Action, ActionRequestQueue, NotesFillScope},
    state::SolvabilityState,
    ui::icon,
};

pub fn show_new_game_confirm(ctx: &Context, action_queue: &mut ActionRequestQueue) {
    let modal = Modal::new(Id::new("new_game_confirm")).show(ctx, |ui| {
        ui.heading("New Game?");
        ui.add_space(4.0);
        ui.label("Start a new game? Current progress will be lost.");
        ui.add_space(8.0);

        Sides::new().show(
            ui,
            |_ui| {},
            |ui| {
                let new_game = ui.button(format!("{} New Game", icon::CHECK));
                if ui.memory(|memory| memory.focused().is_none()) {
                    new_game.request_focus();
                }
                if new_game.clicked() {
                    action_queue.request(Action::StartNewGame);
                    ui.close();
                }
                if ui.button(format! {"{} Cancel", icon::CANCEL}).clicked() {
                    ui.close();
                }
            },
        );
    });
    if modal.should_close() {
        action_queue.request(Action::CloseModal);
    }
}

pub fn show_reset_current_puzzle_confirm(ctx: &Context, action_queue: &mut ActionRequestQueue) {
    let modal = Modal::new(Id::new("reset_current_puzzle_confirm")).show(ctx, |ui| {
        ui.heading("Reset Puzzle?");
        ui.add_space(4.0);
        ui.label("Clear all your inputs and return to the initial puzzle state?");
        ui.add_space(8.0);

        Sides::new().show(
            ui,
            |_ui| {},
            |ui| {
                let reset = ui.button(format!("{} Reset", icon::CHECK));
                if ui.memory(|memory| memory.focused().is_none()) {
                    reset.request_focus();
                }
                if reset.clicked() {
                    action_queue.request(Action::ResetCurrentPuzzle);
                    ui.close();
                }
                if ui.button(format! {"{} Cancel", icon::CANCEL}).clicked() {
                    ui.close();
                }
            },
        );
    });
    if modal.should_close() {
        action_queue.request(Action::CloseModal);
    }
}

pub fn show_solvability(
    ctx: &Context,
    action_queue: &mut ActionRequestQueue,
    state: &SolvabilityState,
) {
    let modal = Modal::new(Id::new("solvability_result")).show(ctx, |ui| {
        match state {
            SolvabilityState::Inconsistent => {
                ui.heading("Board Inconsistent");
                ui.add_space(4.0);
                ui.label("A conflict or a no-candidate cell was detected. We recommend undoing to the last consistent state.");
                ui.add_space(8.0);

                Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        ui.add_enabled(false, Button::new(format!("{} Undo (coming soon)", icon::ARROW_UNDO)));
                        if ui.button(format! {"{} Cancel", icon::CANCEL}).clicked() {
                            ui.close();
                        }
                    },
                );
            },
            SolvabilityState::NoSolution => {
                ui.heading("No Solution Found");
                ui.add_space(4.0);
                ui.label("No solution exists from the current state. We recommend undoing to the last solvable state.");
                ui.add_space(8.0);

                Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        ui.add_enabled(false, Button::new(format!("{} Undo (coming soon)", icon::ARROW_UNDO)));
                        if ui.button(format! {"{} Cancel", icon::CANCEL}).clicked() {
                            ui.close();
                        }
                    },
                );
            },
            SolvabilityState::Solvable { with_user_notes: true, stats: _stats } => {
                ui.heading("Solvable");
                ui.add_space(4.0);
                ui.label("A solution is still possible from the current state.");
                ui.add_space(8.0);

                Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button(format! {"{} OK", icon::CHECK}).clicked() {
                            ui.close();
                        }
                    },
                );
            },
            SolvabilityState::Solvable { with_user_notes: false, stats: _stats } => {
                ui.heading("Notes May Be Incorrect");
                ui.add_space(4.0);
                ui.label("A solution exists when ignoring notes. Rebuild candidates now?");
                ui.add_space(8.0);

                Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button(format! {"{} Rebuild", icon::CHECK}).clicked() {
                            action_queue.request(Action::AutoFillNotes { scope: NotesFillScope::AllCells });
                            ui.close();
                        }
                        if ui.button(format! {"{} Cancel", icon::CANCEL}).clicked() {
                            ui.close();
                        }
                    },
                );
            },
        }
    });
    if modal.should_close() {
        action_queue.request(Action::CloseModal);
    }
}
