use numelace_core::{Digit, Position};
use numelace_game::{GameError, RuleCheckPolicy};

use crate::{
    action::{Action, ActionRequestQueue, MoveDirection},
    game_factory,
    state::{AppState, GhostType, InputMode, UiState},
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionEffect {
    pub state_save_requested: bool,
    pub theme_changed: bool,
}

#[derive(Debug)]
struct ActionContext<'a> {
    app_state: &'a mut AppState,
    ui_state: &'a mut UiState,
    effect: &'a mut ActionEffect,
}

pub fn handle_all(
    app_state: &mut AppState,
    ui_state: &mut UiState,
    effect: &mut ActionEffect,
    action_queue: &mut ActionRequestQueue,
) {
    for action in action_queue.take_all() {
        handle(app_state, ui_state, effect, action);
    }
}

pub fn handle(
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
        Action::SelectCell(pos) => {
            ctx.app_state.selected_cell = Some(pos);
        }
        Action::ClearSelection => {
            ctx.app_state.selected_cell = None;
        }
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
        Action::ToggleInputMode => {
            ctx.app_state.input_mode.toggle();
        }
        Action::RequestDigit { digit, swap } => {
            ctx.request_digit(digit, swap);
        }
        Action::ClearCell => {
            ctx.clear_cell();
        }
        Action::Undo => {
            push_history_if_changed = false;
            ctx.ui_state.undo(ctx.app_state);
        }
        Action::Redo => {
            push_history_if_changed = false;
            ctx.ui_state.redo(ctx.app_state);
        }
        Action::RequestNewGameConfirm => {
            ctx.ui_state.show_new_game_confirm_dialogue = true;
        }
        Action::CloseNewGameConfirm => {
            ctx.ui_state.show_new_game_confirm_dialogue = false;
        }
        Action::StartNewGame => {
            push_history_if_changed = false;
            ctx.start_new_game();
        }
        Action::UpdateSettings(settings) => {
            ctx.app_state.settings = settings;
            ctx.effect.theme_changed = true;
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
                        self.app_state.game.toggle_digit(pos, digit, &options)
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

    fn start_new_game(&mut self) {
        self.app_state.game = game_factory::generate_random_game();
        self.app_state.selected_cell = None;
        self.ui_state.reset_history(self.app_state);
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{Digit, DigitGrid, Position};
    use numelace_game::{CellState, Game};

    use super::{ActionEffect, handle};
    use crate::{
        DEFAULT_MAX_HISTORY_LENGTH,
        action::Action,
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
        Game::from_problem_filled_notes(&problem, &filled, &notes).unwrap()
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
    fn close_new_game_confirm_clears_flag() {
        let mut app_state = AppState::new(fixed_game());
        let mut ui_state = UiState::new(DEFAULT_MAX_HISTORY_LENGTH, &app_state);
        ui_state.show_new_game_confirm_dialogue = true;
        let mut effect = ActionEffect::default();

        handle(
            &mut app_state,
            &mut ui_state,
            &mut effect,
            Action::CloseNewGameConfirm,
        );

        assert!(!ui_state.show_new_game_confirm_dialogue);
        assert!(effect.state_save_requested);
    }
}
