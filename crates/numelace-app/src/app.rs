//! Numelace desktop application UI.
//!
//! # Design Notes
//! - Desktop-focused MVP with a 9x9 grid and clear 3x3 boundaries.
//! - Keyboard-driven input (digits, arrows, delete/backspace) with mouse selection.
//! - Status display derived from `Game::is_solved()`.
//!
//! # Future Enhancements
//! - Candidate marks, undo/redo, hints, mistake detection.
//! - Save/load, timer/statistics, and web/WASM support.

use std::time::Duration;

use eframe::{
    App, CreationContext, Frame, Storage,
    egui::{CentralPanel, Context, Id, Modal, Spinner},
};
use numelace_game::Game;

use crate::{
    DEFAULT_MAX_HISTORY_LENGTH,
    action::{self, ActionRequestQueue, ModalRequest},
    flow_executor::SpinnerKind,
    game_factory,
    persistence::storage,
    state::{AppState, UiState},
    ui, view_model_builder, worker,
};

#[derive(Debug)]
pub struct NumelaceApp {
    app_state: AppState,
    ui_state: UiState,
}

impl NumelaceApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let _ = worker::warm_up();
        let app_state = cc.storage.and_then(storage::load_state).unwrap_or_else(|| {
            let puzzle = game_factory::generate_random_puzzle();
            AppState::new_with_settings_applied(Game::new(puzzle))
        });
        let ui_state = UiState::new(DEFAULT_MAX_HISTORY_LENGTH, &app_state);
        Self {
            app_state,
            ui_state,
        }
    }

    fn apply_persistence(&mut self, frame: &mut Frame) {
        if self.app_state.is_dirty()
            && let Some(storage) = frame.storage_mut()
        {
            self.save(storage);
            self.app_state.clear_dirty();
        }
    }
}

impl App for NumelaceApp {
    fn save(&mut self, storage: &mut dyn Storage) {
        storage::save_state(storage, &self.app_state);
    }

    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let mut action_queue = ActionRequestQueue::default();

        self.ui_state.flow.poll(&mut action_queue);
        action::handler::handle_all(&mut self.app_state, &mut self.ui_state, &mut action_queue);

        if self.ui_state.active_modal.is_none() && self.ui_state.flow.active_spinner().is_none() {
            ctx.input(|i| {
                ui::input::handle_input(i, &mut action_queue);
                action::handler::handle_all(
                    &mut self.app_state,
                    &mut self.ui_state,
                    &mut action_queue,
                );
            });
        }

        let game_screen_vm =
            view_model_builder::build_game_screen_view_model(&self.app_state, &self.ui_state);

        CentralPanel::default().show(ctx, |ui| {
            ui::game_screen::show(ui, &game_screen_vm, &mut action_queue);
        });

        if let Some(modal_request) = &mut self.ui_state.active_modal {
            match modal_request {
                ModalRequest::NewGameConfirm(responder) => {
                    ui::dialogs::show_new_game_confirm(ctx, &mut action_queue, responder);
                }
                ModalRequest::ResetCurrentPuzzleConfirm(responder) => {
                    ui::dialogs::show_reset_current_puzzle_confirm(
                        ctx,
                        &mut action_queue,
                        responder,
                    );
                }
                ModalRequest::Settings => {
                    let settings_vm =
                        view_model_builder::build_settings_view_model(&self.app_state);
                    ui::settings::show(ctx, &settings_vm, &mut action_queue);
                }
                ModalRequest::CheckSolvabilityResult { state, responder } => {
                    ui::dialogs::show_solvability(ctx, &mut action_queue, state, responder);
                }
            }
        }

        if let Some(spinner) = self.ui_state.flow.active_spinner() {
            ctx.request_repaint();
            match spinner {
                SpinnerKind::NewGame => {
                    Modal::new(Id::new("generating_new_game")).show(ctx, |ui| {
                        ui.heading("Generating...");
                        ui.add(Spinner::new());
                        ui.label("Generating new game...");
                    });
                }
                SpinnerKind::CheckSolvability => {
                    Modal::new(Id::new("checking_solvability")).show(ctx, |ui| {
                        ui.heading("Checking...");
                        ui.add(Spinner::new());
                        ui.label("Checking solvability...");
                    });
                }
            }
        }

        action::handler::handle_all(&mut self.app_state, &mut self.ui_state, &mut action_queue);

        self.apply_persistence(frame);
    }
}
