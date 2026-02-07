use numelace_core::{Digit, Position};
use numelace_game::{GameError, RuleCheckPolicy};

use crate::{
    action::{
        Action, ActionRequestQueue, ModalResponse, MoveDirection, NotesFillScope, WorkRequestAction,
    },
    async_work::{WorkResponse, work_actions},
    flow::{check_solvability_flow, new_game_flow},
    state::{AppState, GhostType, InputMode, UiState},
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActionEffect {
    pub(crate) state_save_requested: bool,
}

#[derive(Debug)]
struct ActionContext<'a> {
    app_state: &'a mut AppState,
    ui_state: &'a mut UiState,
    effect: &'a mut ActionEffect,
}

pub(crate) fn handle_all(
    app_state: &mut AppState,
    ui_state: &mut UiState,
    effect: &mut ActionEffect,
    action_queue: &mut ActionRequestQueue,
) {
    for action in action_queue.take_all() {
        handle(app_state, ui_state, effect, action);
    }
}

pub(crate) fn handle(
    app_state: &mut AppState,
    ui_state: &mut UiState,
    effect: &mut ActionEffect,
    action: Action,
) {
    const DEFAULT_POSITION: Position = Position::new(0, 0);

    let mut ctx = ActionContext {
        app_state,
        ui_state,
        effect,
    };

    let game_snapshot_before = ctx.app_state.game.clone();
    let mut push_history_if_changed = true;

    // For now, mark the app state as dirty for every action to simplify persistence; UI-only changes are acceptable to save.
    ctx.effect.state_save_requested = true;

    ctx.ui_state.conflict_ghost = None;

    match action {
        Action::SelectCell(pos) => ctx.app_state.selected_cell = Some(pos),
        Action::ClearSelection => ctx.app_state.selected_cell = None,
        Action::MoveSelection(move_direction) => {
            let pos = ctx.app_state.selected_cell.get_or_insert(DEFAULT_POSITION);
            let new_pos = match move_direction {
                MoveDirection::Up => pos.up(),
                MoveDirection::Down => pos.down(),
                MoveDirection::Left => pos.left(),
                MoveDirection::Right => pos.right(),
            };
            if let Some(new_pos) = new_pos {
                *pos = new_pos;
            }
        }
        Action::ToggleInputMode => ctx.app_state.input_mode.toggle(),
        Action::RequestDigit { digit, swap } => ctx.request_digit(digit, swap),
        Action::ClearCell => ctx.clear_cell(),
        Action::AutoFillNotes { scope } => ctx.auto_fill_notes(scope),
        Action::CheckSolvability => ctx.check_solvability(),
        Action::Undo => {
            push_history_if_changed = false;
            ctx.ui_state.undo(ctx.app_state);
        }
        Action::Redo => {
            push_history_if_changed = false;
            ctx.ui_state.redo(ctx.app_state);
        }
        Action::OpenModal(modal_request) => {
            ctx.ui_state.active_modal = Some(modal_request.modal);
            ctx.ui_state.modal_responder = modal_request.responder;
        }
        Action::CloseModal => {
            ctx.ui_state.active_modal = None;
            ctx.ui_state.modal_responder = None;
        }
        Action::StartNewGameFlow => {
            push_history_if_changed = false;
            ctx.start_new_game_flow();
        }
        Action::ModalResponse(response) => {
            push_history_if_changed = false;
            ctx.handle_modal_response(response);
        }
        Action::StartWork(request_action) => {
            push_history_if_changed = false;
            ctx.request_work(request_action);
        }
        Action::ResetCurrentPuzzle => {
            push_history_if_changed = false;
            ctx.reset_current_puzzle();
        }
        Action::ApplyWorkResponse(response) => {
            push_history_if_changed = false;
            ctx.apply_work_response(response);
        }
        Action::UpdateSettings(settings) => {
            ctx.app_state.settings = settings;
        }
    }

    if push_history_if_changed && ctx.app_state.game != game_snapshot_before {
        ctx.ui_state.push_history(ctx.app_state);
    }
}

impl ActionContext<'_> {
    fn request_digit(&mut self, digit: Digit, swap: bool) {
        if let Some(pos) = self.app_state.selected_cell {
            match self.app_state.input_mode.swapped(swap) {
                InputMode::Fill => {
                    let options = self.app_state.input_digit_options();
                    if let Err(GameError::ConflictingDigit) =
                        self.app_state.game.set_digit(pos, digit, &options)
                    {
                        assert_eq!(self.app_state.rule_check_policy(), RuleCheckPolicy::Strict);
                        self.ui_state.conflict_ghost = Some((pos, GhostType::Digit(digit)));
                    }
                }
                InputMode::Notes => {
                    let policy = self.app_state.rule_check_policy();
                    if let Err(GameError::ConflictingDigit) =
                        self.app_state.game.toggle_note(pos, digit, policy)
                    {
                        assert_eq!(policy, RuleCheckPolicy::Strict);
                        self.ui_state.conflict_ghost = Some((pos, GhostType::Note(digit)));
                    }
                }
            }
        }
    }

    fn clear_cell(&mut self) {
        if let Some(pos) = self.app_state.selected_cell {
            let _ = self.app_state.game.clear_cell(pos);
        }
    }

    fn request_work(&mut self, request_action: WorkRequestAction) {
        let _ = work_actions::request_work(request_action, self.ui_state);
    }

    fn start_new_game_flow(&mut self) {
        if !self.ui_state.flow.is_idle()
            || crate::async_work::work_flow::WorkFlow::is_work_in_flight(&self.ui_state.work)
        {
            return;
        }
        let handle = self.ui_state.flow.handle();
        self.ui_state.flow.spawn(new_game_flow(handle));
    }

    fn handle_modal_response(&mut self, response: ModalResponse) {
        if let Some(responder) = self.ui_state.modal_responder.take() {
            let _ = responder.send(response);
        }
    }

    fn apply_work_response(&mut self, response: WorkResponse) {
        if let Some(responder) = self.ui_state.work.work_responder.take() {
            let _ = responder.send(response.clone());
        }
        work_actions::apply_work_response(self.app_state, self.ui_state, response);
    }

    fn reset_current_puzzle(&mut self) {
        self.app_state.reset_current_puzzle_state();
        self.ui_state.reset_history(self.app_state);
    }

    fn auto_fill_notes(&mut self, scope: NotesFillScope) {
        match scope {
            NotesFillScope::Cell => {
                if let Some(pos) = self.app_state.selected_cell {
                    let _ = self.app_state.game.auto_fill_cell_notes(pos);
                }
            }
            NotesFillScope::AllCells => {
                self.app_state.game.auto_fill_notes_all_cells();
            }
        }
    }

    fn check_solvability(&mut self) {
        if !self.ui_state.flow.is_idle()
            || crate::async_work::work_flow::WorkFlow::is_work_in_flight(&self.ui_state.work)
        {
            return;
        }

        let handle = self.ui_state.flow.handle();
        let request = work_actions::build_solvability_request(&self.app_state.game);
        self.ui_state
            .flow
            .spawn(check_solvability_flow(handle, request));
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{Digit, DigitGrid, Position};
    use numelace_game::{CellState, Game};

    use super::{ActionEffect, handle};
    use crate::{
        DEFAULT_MAX_HISTORY_LENGTH,
        action::{Action, NotesFillScope},
        state::{AppState, GhostType, ModalKind, UiState},
    };

    fn fixed_game() -> Game {
        let problem: DigitGrid = "\
.1.......\
.........\
.........\
.........\
.........\
.........\
.........\
.........\
.........\
"
        .parse()
        .unwrap();
        let solution: DigitGrid =
            "185362947793148526246795183564239871931874265827516394318427659672951438459683712"
                .parse()
                .unwrap();
        let filled: DigitGrid = "\
.........\
.........\
.........\
.........\
.........\
.........\
.........\
.........\
.........\
"
        .parse()
        .unwrap();
        let notes = [[0u16; 9]; 9];
        Game::from_problem_filled_notes(&problem, &solution, &filled, &notes).unwrap()
    }

    #[test]
    fn conflicting_digit_sets_ghost_and_requests_save() {
        let mut app_state = AppState::new(fixed_game());
        app_state.selected_cell = Some(Position::new(0, 0));
        app_state.settings.assist.block_rule_violations = true;

        let mut ui_state = UiState::new(DEFAULT_MAX_HISTORY_LENGTH, &app_state);
        let mut effect = ActionEffect::default();

        handle(
            &mut app_state,
            &mut ui_state,
            &mut effect,
            Action::RequestDigit {
                digit: Digit::D1,
                swap: false,
            },
        );

        assert!(effect.state_save_requested);
        assert_eq!(
            ui_state.conflict_ghost,
            Some((Position::new(0, 0), GhostType::Digit(Digit::D1)))
        );
        assert!(matches!(
            app_state.game.cell(Position::new(0, 0)),
            CellState::Empty
        ));
    }

    #[test]
    fn auto_fill_cell_without_selection_is_noop() {
        let mut app_state = AppState::new(fixed_game());
        let mut ui_state = UiState::new(DEFAULT_MAX_HISTORY_LENGTH, &app_state);
        let mut effect = ActionEffect::default();
        let before = app_state.game.clone();

        handle(
            &mut app_state,
            &mut ui_state,
            &mut effect,
            Action::AutoFillNotes {
                scope: NotesFillScope::Cell,
            },
        );

        assert_eq!(app_state.game, before);
    }

    #[test]
    fn reset_current_puzzle_auto_fills_notes_when_enabled() {
        let mut app_state = AppState::new(fixed_game());
        app_state
            .settings
            .assist
            .notes
            .auto_fill_notes_on_new_or_reset = true;
        let mut ui_state = UiState::new(DEFAULT_MAX_HISTORY_LENGTH, &app_state);
        let mut effect = ActionEffect::default();

        handle(
            &mut app_state,
            &mut ui_state,
            &mut effect,
            Action::ResetCurrentPuzzle,
        );

        let any_notes = Position::ALL
            .into_iter()
            .any(|pos| app_state.game.cell(pos).is_notes());
        assert!(any_notes);
    }

    #[test]
    fn same_digit_request_does_not_add_history_entry() {
        let mut app_state = AppState::new(fixed_game());
        app_state.selected_cell = Some(Position::new(0, 0));
        let mut ui_state = UiState::new(DEFAULT_MAX_HISTORY_LENGTH, &app_state);
        let mut effect = ActionEffect::default();

        handle(
            &mut app_state,
            &mut ui_state,
            &mut effect,
            Action::RequestDigit {
                digit: Digit::D2,
                swap: false,
            },
        );

        handle(
            &mut app_state,
            &mut ui_state,
            &mut effect,
            Action::RequestDigit {
                digit: Digit::D2,
                swap: false,
            },
        );

        assert!(ui_state.can_undo());
        assert!(ui_state.undo(&mut app_state));
        assert!(!ui_state.can_undo());
        assert!(matches!(
            app_state.game.cell(Position::new(0, 0)),
            CellState::Empty
        ));
    }

    #[test]
    fn close_new_game_confirm_clears_flag() {
        let mut app_state = AppState::new(fixed_game());
        let mut ui_state = UiState::new(DEFAULT_MAX_HISTORY_LENGTH, &app_state);
        ui_state.active_modal = Some(ModalKind::NewGameConfirm);
        let mut effect = ActionEffect::default();

        handle(
            &mut app_state,
            &mut ui_state,
            &mut effect,
            Action::CloseModal,
        );

        assert!(ui_state.active_modal.is_none());
        assert!(effect.state_save_requested);
    }
}
