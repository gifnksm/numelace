use numelace_core::Position;
use numelace_game::{Game, GameError, RuleCheckPolicy};

use crate::{
    action::{
        Action, ActionRequestQueue, AppAction, BoardMutationAction, FlowAction, HistoryAction,
        InputModeAction, NotesFillScope, PuzzleLifecycleAction, SelectionAction, SettingsAction,
        StateQueryAction, UiAction,
    },
    flow,
    state::{AppState, AppStateAccess, GhostType, InputMode, UiState},
};

#[derive(Debug)]
struct ActionContext<'a> {
    app_state: AppStateAccess<'a>,
    ui_state: &'a mut UiState,
}

pub(crate) fn handle_all(
    app_state: &mut AppState,
    ui_state: &mut UiState,
    action_queue: &mut ActionRequestQueue,
) {
    for action in action_queue.take_all() {
        handle(app_state, ui_state, action);
    }
}

pub(crate) fn handle(app_state: &mut AppState, ui_state: &mut UiState, action: Action) {
    let mut ctx = ActionContext {
        app_state: app_state.access(),
        ui_state,
    };
    ctx.ui_state.conflict_ghost = None;
    ctx.handle_action(action);
}

impl ActionContext<'_> {
    fn handle_action(&mut self, action: Action) {
        match action {
            Action::App(action) => action.execute(self.app_state.as_mut(), self.ui_state),
            Action::Ui(action) => action.execute(self.ui_state),
            Action::Flow(action) => action.execute(self.app_state.as_ref(), self.ui_state),
        }
    }
}

impl AppAction {
    fn execute(self, app_state: &mut AppState, ui_state: &mut UiState) {
        match self {
            AppAction::BoardMutation(action) => {
                action.execute(app_state, ui_state);
            }
            AppAction::PuzzleLifecycle(action) => {
                action.execute(app_state, ui_state);
            }
            AppAction::History(action) => action.execute(app_state, ui_state),
            AppAction::StateQuery(action) => action.execute(app_state, ui_state),
            AppAction::Selection(action) => action.execute(app_state),
            AppAction::InputMode(action) => action.execute(app_state),
            AppAction::Settings(action) => action.execute(app_state),
        }
    }
}

impl BoardMutationAction {
    fn execute(self, app_state: &mut AppState, ui_state: &mut UiState) {
        let game_snapshot = app_state.game.clone();
        match self {
            BoardMutationAction::RequestDigit { digit, swap } => {
                if let Some(pos) = app_state.selected_cell {
                    match app_state.input_mode.swapped(swap) {
                        InputMode::Fill => {
                            let options = app_state.input_digit_options();
                            if let Err(GameError::ConflictingDigit) =
                                app_state.game.set_digit(pos, digit, &options)
                            {
                                assert_eq!(app_state.rule_check_policy(), RuleCheckPolicy::Strict);
                                ui_state.conflict_ghost = Some((pos, GhostType::Digit(digit)));
                            }
                        }
                        InputMode::Notes => {
                            let policy = app_state.rule_check_policy();
                            if let Err(GameError::ConflictingDigit) =
                                app_state.game.toggle_note(pos, digit, policy)
                            {
                                assert_eq!(policy, RuleCheckPolicy::Strict);
                                ui_state.conflict_ghost = Some((pos, GhostType::Note(digit)));
                            }
                        }
                    }
                }
            }
            BoardMutationAction::ClearCell => {
                if let Some(pos) = app_state.selected_cell {
                    let _ = app_state.game.clear_cell(pos);
                }
            }
            BoardMutationAction::AutoFillNotes { scope } => match scope {
                NotesFillScope::Cell => {
                    if let Some(pos) = app_state.selected_cell {
                        let _ = app_state.game.auto_fill_cell_notes(pos);
                    }
                }
                NotesFillScope::AllCells => {
                    app_state.game.auto_fill_notes_all_cells();
                }
            },
            BoardMutationAction::ResetInputs => {
                for pos in Position::ALL {
                    let _ = app_state.game.clear_cell(pos);
                }
                app_state.apply_new_game_settings();
            }
            BoardMutationAction::ApplyTechniqueStep(step) => {
                let options = &app_state.input_digit_options();
                let _ = app_state.game.apply_technique_step(step.as_ref(), options);
            }
        }
        if app_state.game != game_snapshot {
            ui_state.hint_state = None;
            app_state.push_history();
        }
    }
}

impl PuzzleLifecycleAction {
    fn execute(self, app_state: &mut AppState, ui_state: &mut UiState) {
        match self {
            PuzzleLifecycleAction::StartNewGame(puzzle) => {
                let game = Game::new(puzzle);
                app_state.game = game;
                app_state.selected_cell = None;
                app_state.apply_new_game_settings();
                app_state.reset_history();
                ui_state.hint_state = None;
            }
        }
    }
}

impl HistoryAction {
    fn execute(self, app_state: &mut AppState, ui_state: &mut UiState) {
        ui_state.hint_state = None;
        match self {
            HistoryAction::Undo => {
                app_state.undo();
            }
            HistoryAction::UndoSteps(steps) => {
                app_state.undo_steps(steps);
            }
            HistoryAction::Redo => {
                app_state.redo();
            }
        }
    }
}

impl StateQueryAction {
    fn execute(self, app_state: &mut AppState, _ui_state: &mut UiState) {
        match self {
            StateQueryAction::BuildUndoGames { responder } => {
                let games = app_state.build_undo_games();
                let _ = responder.send(games);
            }
        }
    }
}

impl SelectionAction {
    fn execute(self, app_state: &mut AppState) {
        const DEFAULT_POSITION: Position = Position::new(0, 0);

        match self {
            SelectionAction::SelectCell(pos) => app_state.selected_cell = Some(pos),
            SelectionAction::ClearSelection => app_state.selected_cell = None,
            SelectionAction::MoveSelection(move_direction) => {
                let pos = app_state.selected_cell.get_or_insert(DEFAULT_POSITION);
                if let Some(new_pos) = move_direction.apply_to(*pos) {
                    *pos = new_pos;
                }
            }
        }
    }
}

impl InputModeAction {
    fn execute(self, app_state: &mut AppState) {
        match self {
            InputModeAction::ToggleInputMode => app_state.input_mode.toggle(),
        }
    }
}

impl SettingsAction {
    fn execute(self, app_state: &mut AppState) {
        match self {
            SettingsAction::UpdateSettings(settings) => {
                app_state.settings = settings;
            }
        }
    }
}

impl UiAction {
    fn execute(self, ui_state: &mut UiState) {
        match self {
            UiAction::OpenModal(modal_request) => {
                ui_state.active_modal = Some(modal_request);
            }
            UiAction::CloseModal => {
                ui_state.active_modal = None;
            }
            UiAction::StartSpinner { id, kind } => {
                ui_state.spinner_state.start(id, kind);
            }
            UiAction::StopSpinner { id } => {
                ui_state.spinner_state.stop(id);
            }
            UiAction::SetHintState(hint_state) => {
                ui_state.hint_state = hint_state;
            }
            UiAction::ClearHintState => {
                ui_state.hint_state = None;
            }
        }
    }
}

impl FlowAction {
    fn execute(self, app_state: &AppState, ui_state: &mut UiState) {
        match self {
            FlowAction::StartNewGame => {
                flow::tasks::spawn_new_game_flow(&mut ui_state.executor);
            }
            FlowAction::ResetInputs => {
                flow::tasks::spawn_reset_inputs_flow(&mut ui_state.executor);
            }
            FlowAction::CheckSolvability => {
                flow::tasks::spawn_check_solvability_flow(&mut ui_state.executor, &app_state.game);
            }
            FlowAction::Hint => {
                flow::tasks::spawn_hint_flow(
                    &mut ui_state.executor,
                    &app_state.game,
                    ui_state.hint_state.clone(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{Digit, DigitGrid, Position};
    use numelace_game::{CellState, Game};

    use super::handle;
    use crate::{
        action::{BoardMutationAction, ConfirmKind, ModalRequest, NotesFillScope, UiAction},
        state::{AppState, GhostType, UiState},
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

        let mut ui_state = UiState::new();

        handle(
            &mut app_state,
            &mut ui_state,
            BoardMutationAction::RequestDigit {
                digit: Digit::D1,
                swap: false,
            }
            .into(),
        );
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
        let mut ui_state = UiState::new();
        let before = app_state.game.clone();

        handle(
            &mut app_state,
            &mut ui_state,
            BoardMutationAction::AutoFillNotes {
                scope: NotesFillScope::Cell,
            }
            .into(),
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
        let mut ui_state = UiState::new();

        handle(
            &mut app_state,
            &mut ui_state,
            BoardMutationAction::ResetInputs.into(),
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
        let mut ui_state = UiState::new();

        handle(
            &mut app_state,
            &mut ui_state,
            BoardMutationAction::RequestDigit {
                digit: Digit::D2,
                swap: false,
            }
            .into(),
        );

        handle(
            &mut app_state,
            &mut ui_state,
            BoardMutationAction::RequestDigit {
                digit: Digit::D2,
                swap: false,
            }
            .into(),
        );

        assert!(app_state.can_undo());
        assert!(app_state.undo());
        assert!(!app_state.can_undo());
        assert!(matches!(
            app_state.game.cell(Position::new(0, 0)),
            CellState::Empty
        ));
    }

    #[test]
    fn close_new_game_confirm_clears_flag() {
        let mut app_state = AppState::new(fixed_game());
        let mut ui_state = UiState::new();
        ui_state.active_modal = Some(ModalRequest::Confirm {
            kind: ConfirmKind::NewGame,
            responder: None,
        });

        handle(&mut app_state, &mut ui_state, UiAction::CloseModal.into());

        assert!(ui_state.active_modal.is_none());
    }
}
