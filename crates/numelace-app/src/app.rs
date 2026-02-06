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

use crate::{
    DEFAULT_MAX_HISTORY_LENGTH,
    action::ActionRequestQueue,
    action_handler::{self, ActionEffect},
    async_work,
    async_work::work_flow::WorkFlow,
    game_factory,
    persistence::storage,
    state::{AppState, ModalKind, UiState},
    ui, view_model_builder,
};

#[derive(Debug)]
pub struct NumelaceApp {
    app_state: AppState,
    ui_state: UiState,
}

impl NumelaceApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let _ = async_work::warm_up();
        let app_state = cc.storage.and_then(storage::load_state).unwrap_or_else(|| {
            AppState::new_with_settings_applied(game_factory::generate_random_game())
        });
        let ui_state = UiState::new(DEFAULT_MAX_HISTORY_LENGTH, &app_state);
        Self {
            app_state,
            ui_state,
        }
    }

    fn apply_effect(&mut self, frame: &mut Frame, effect: ActionEffect) {
        if effect.state_save_requested
            && let Some(storage) = frame.storage_mut()
        {
            self.save(storage);
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
        let mut effect = ActionEffect::default();
        let mut action_queue = ActionRequestQueue::default();

        let work_flow = WorkFlow;
        work_flow.poll_and_queue(&mut self.ui_state.work, &mut action_queue);

        if self.ui_state.active_modal.is_none() && !WorkFlow::is_work_in_flight(&self.ui_state.work)
        {
            ctx.input(|i| {
                ui::input::handle_input(i, &mut action_queue);
                action_handler::handle_all(
                    &mut self.app_state,
                    &mut self.ui_state,
                    &mut effect,
                    &mut action_queue,
                );
            });
        }

        let game_screen_vm =
            view_model_builder::build_game_screen_view_model(&self.app_state, &self.ui_state);

        CentralPanel::default().show(ctx, |ui| {
            ui::game_screen::show(ui, &game_screen_vm, &mut action_queue);
        });

        if let Some(modal) = &self.ui_state.active_modal {
            match modal {
                ModalKind::NewGameConfirm => {
                    ui::dialogs::show_new_game_confirm(ctx, &mut action_queue);
                }
                ModalKind::ResetCurrentPuzzleConfirm => {
                    ui::dialogs::show_reset_current_puzzle_confirm(ctx, &mut action_queue);
                }
                ModalKind::Settings => {
                    let settings_vm =
                        view_model_builder::build_settings_view_model(&self.app_state);
                    ui::settings::show(ctx, &settings_vm, &mut action_queue);
                }
                ModalKind::CheckSolvabilityResult(state) => {
                    ui::dialogs::show_solvability(ctx, &mut action_queue, state);
                }
            }
        }

        if WorkFlow::is_work_in_flight(&self.ui_state.work) {
            ctx.request_repaint();
            if WorkFlow::is_new_game_in_flight(&self.ui_state.work) {
                Modal::new(Id::new("generating_new_game")).show(ctx, |ui| {
                    ui.heading("Generating...");
                    ui.add(Spinner::new());
                    ui.label("Generating new game...");
                });
            } else if WorkFlow::is_solvability_check_in_flight(&self.ui_state.work) {
                Modal::new(Id::new("checking_solvability")).show(ctx, |ui| {
                    ui.heading("Checking...");
                    ui.add(Spinner::new());
                    ui.label("Checking solvability...");
                });
            }
        }

        action_handler::handle_all(
            &mut self.app_state,
            &mut self.ui_state,
            &mut effect,
            &mut action_queue,
        );

        self.apply_effect(frame, effect);
    }
}
