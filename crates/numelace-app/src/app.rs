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
    egui::{CentralPanel, Context},
};
use numelace_game::Game;

use crate::{
    action::{self, ActionRequestQueue, FlowAction},
    persistence::storage,
    state::{AppState, UiState},
    ui, view_model_builder, worker,
};

#[derive(Debug)]
pub struct NumelaceApp {
    app_state: AppState,
    ui_state: UiState,
}

const MAX_ACTION_HANDLING_ITERATIONS: usize = 10;

impl NumelaceApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let _ = worker::warm_up();
        let app_state = cc
            .storage
            .and_then(storage::load_state)
            .unwrap_or_else(|| AppState::new_with_settings_applied(Game::new_empty()));
        let ui_state = UiState::new();
        Self {
            app_state,
            ui_state,
        }
    }

    fn poll_and_handle_actions(&mut self, action_queue: &mut ActionRequestQueue) {
        self.ui_state.executor.poll(action_queue);
        for _ in 0..MAX_ACTION_HANDLING_ITERATIONS {
            if action_queue.is_empty() {
                break;
            }
            action::handler::handle_all(&mut self.app_state, &mut self.ui_state, action_queue);
            self.ui_state.executor.poll(action_queue);
        }
    }

    fn handle_actions(&mut self, action_queue: &mut ActionRequestQueue) {
        for _ in 0..MAX_ACTION_HANDLING_ITERATIONS {
            if action_queue.is_empty() {
                break;
            }
            action::handler::handle_all(&mut self.app_state, &mut self.ui_state, action_queue);
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

        if !self.app_state.game.is_initialized() && !self.ui_state.requested_initial_new_game {
            action_queue.request(FlowAction::StartNewGame.into());
            self.ui_state.requested_initial_new_game = true;
        }

        self.poll_and_handle_actions(&mut action_queue);

        let allow_input =
            self.ui_state.active_modal.is_none() && !self.ui_state.spinner_state.is_active();
        let base_input_mode = self.app_state.input_mode;
        let input_context = ctx.input(|i| {
            let context = ui::input::build_input_context(i, allow_input, base_input_mode);
            if allow_input {
                ui::input::handle_input(i, &context, &mut action_queue);
                self.handle_actions(&mut action_queue);
            }
            context
        });

        let game_screen_vm = view_model_builder::build_game_screen_view_model(
            &self.app_state,
            &self.ui_state,
            &input_context,
        );

        CentralPanel::default().show(ctx, |ui| {
            ui::game_screen::show(ui, &game_screen_vm, &mut action_queue);
        });

        if let Some(modal_request) = &mut self.ui_state.active_modal {
            let new_game_options_vm =
                view_model_builder::build_new_game_options_view_model(&self.app_state);
            let settings_vm = view_model_builder::build_settings_view_model(&self.app_state);
            ui::modal::show(
                ctx,
                &mut action_queue,
                modal_request,
                &new_game_options_vm,
                &settings_vm,
            );
        }

        if let Some(spinner) = self.ui_state.spinner_state.active_kind() {
            ui::spinner::show(ctx, spinner);
        }

        self.poll_and_handle_actions(&mut action_queue);
        self.apply_persistence(frame);
    }
}
