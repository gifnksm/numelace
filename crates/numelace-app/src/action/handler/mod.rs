mod flows;

use numelace_core::{Digit, Position};
use numelace_game::{Game, GameError, RuleCheckPolicy};
use numelace_generator::GeneratedPuzzle;

use crate::{
    action::{Action, ActionRequestQueue, MoveDirection, NotesFillScope},
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
    const DEFAULT_POSITION: Position = Position::new(0, 0);

    let mut ctx = ActionContext {
        app_state: app_state.access(),
        ui_state,
    };

    let game_snapshot_before = ctx.app_state.as_ref().game.clone();
    let mut push_history_if_changed = true;

    ctx.ui_state.conflict_ghost = None;

    match action {
        Action::SelectCell(pos) => ctx.app_state.as_mut().selected_cell = Some(pos),
        Action::ClearSelection => ctx.app_state.as_mut().selected_cell = None,
        Action::MoveSelection(move_direction) => {
            let app_state = ctx.app_state.as_mut();
            let pos = app_state.selected_cell.get_or_insert(DEFAULT_POSITION);
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
        Action::ToggleInputMode => ctx.app_state.as_mut().input_mode.toggle(),
        Action::RequestDigit { digit, swap } => ctx.request_digit(digit, swap),
        Action::ClearCell => ctx.clear_cell(),
        Action::AutoFillNotes { scope } => ctx.auto_fill_notes(scope),
        Action::CheckSolvability => ctx.check_solvability(),
        Action::Undo => {
            push_history_if_changed = false;
            ctx.ui_state.undo(ctx.app_state.as_mut());
        }
        Action::Redo => {
            push_history_if_changed = false;
            ctx.ui_state.redo(ctx.app_state.as_mut());
        }
        Action::OpenModal(modal_request) => {
            ctx.ui_state.active_modal = Some(modal_request);
        }
        Action::CloseModal => {
            ctx.ui_state.active_modal = None;
        }
        Action::NewGameReady(puzzle) => {
            push_history_if_changed = false;
            ctx.apply_new_game_puzzle(puzzle);
        }
        Action::StartSpinner { id, kind } => {
            ctx.ui_state.spinner_state.start(id, kind);
        }
        Action::StopSpinner { id } => {
            ctx.ui_state.spinner_state.stop(id);
        }
        Action::StartNewGameFlow => {
            push_history_if_changed = false;
            ctx.start_new_game_flow();
        }

        Action::ResetCurrentPuzzle => {
            push_history_if_changed = false;
            ctx.reset_current_puzzle();
        }
        Action::UpdateSettings(settings) => {
            ctx.app_state.as_mut().settings = settings;
        }
    }

    if push_history_if_changed && ctx.app_state.as_ref().game != game_snapshot_before {
        ctx.ui_state.push_history(ctx.app_state.as_ref());
    }
}

impl ActionContext<'_> {
    fn request_digit(&mut self, digit: Digit, swap: bool) {
        let app_state = self.app_state.as_mut();
        if let Some(pos) = app_state.selected_cell {
            match app_state.input_mode.swapped(swap) {
                InputMode::Fill => {
                    let options = app_state.input_digit_options();
                    if let Err(GameError::ConflictingDigit) =
                        app_state.game.set_digit(pos, digit, &options)
                    {
                        assert_eq!(app_state.rule_check_policy(), RuleCheckPolicy::Strict);
                        self.ui_state.conflict_ghost = Some((pos, GhostType::Digit(digit)));
                    }
                }
                InputMode::Notes => {
                    let policy = app_state.rule_check_policy();
                    if let Err(GameError::ConflictingDigit) =
                        app_state.game.toggle_note(pos, digit, policy)
                    {
                        assert_eq!(policy, RuleCheckPolicy::Strict);
                        self.ui_state.conflict_ghost = Some((pos, GhostType::Note(digit)));
                    }
                }
            }
        }
    }

    fn clear_cell(&mut self) {
        let app_state = self.app_state.as_mut();
        if let Some(pos) = app_state.selected_cell {
            let _ = app_state.game.clear_cell(pos);
        }
    }

    fn start_new_game_flow(&mut self) {
        flows::spawn_new_game_flow(&mut self.ui_state.flow);
    }

    fn reset_current_puzzle(&mut self) {
        let app_state = self.app_state.as_mut();
        app_state.reset_current_puzzle_state();
        self.ui_state.reset_history(app_state);
    }

    fn apply_new_game_puzzle(&mut self, puzzle: GeneratedPuzzle) {
        let game = Game::new(puzzle);

        let app_state = self.app_state.as_mut();
        app_state.game = game;
        app_state.selected_cell = None;
        app_state.apply_new_game_settings();
        self.ui_state.reset_history(app_state);
    }

    fn auto_fill_notes(&mut self, scope: NotesFillScope) {
        let app_state = self.app_state.as_mut();
        match scope {
            NotesFillScope::Cell => {
                if let Some(pos) = app_state.selected_cell {
                    let _ = app_state.game.auto_fill_cell_notes(pos);
                }
            }
            NotesFillScope::AllCells => {
                app_state.game.auto_fill_notes_all_cells();
            }
        }
    }

    fn check_solvability(&mut self) {
        flows::spawn_check_solvability_flow(&mut self.ui_state.flow, &self.app_state.as_ref().game);
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{Digit, DigitGrid, Position};
    use numelace_game::{CellState, Game};

    use super::handle;
    use crate::{
        DEFAULT_MAX_HISTORY_LENGTH,
        action::{Action, ModalRequest, NotesFillScope},
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

        let mut ui_state = UiState::new(DEFAULT_MAX_HISTORY_LENGTH, &app_state);

        handle(
            &mut app_state,
            &mut ui_state,
            Action::RequestDigit {
                digit: Digit::D1,
                swap: false,
            },
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
        let mut ui_state = UiState::new(DEFAULT_MAX_HISTORY_LENGTH, &app_state);
        let before = app_state.game.clone();

        handle(
            &mut app_state,
            &mut ui_state,
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

        handle(&mut app_state, &mut ui_state, Action::ResetCurrentPuzzle);

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

        handle(
            &mut app_state,
            &mut ui_state,
            Action::RequestDigit {
                digit: Digit::D2,
                swap: false,
            },
        );

        handle(
            &mut app_state,
            &mut ui_state,
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
        ui_state.active_modal = Some(ModalRequest::NewGameConfirm(None));

        handle(&mut app_state, &mut ui_state, Action::CloseModal);

        assert!(ui_state.active_modal.is_none());
    }
}
