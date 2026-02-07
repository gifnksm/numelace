use numelace_core::{Digit, Position};
use numelace_game::{Game, InputDigitOptions, NoteCleanupPolicy, RuleCheckPolicy};

use crate::action::{ModalRequest, WorkResponder};
use crate::async_work::{WorkError, WorkHandle};
use crate::flow::FlowExecutor;
use crate::history::UndoRedoStack;

#[derive(Debug)]
pub(crate) struct AppState {
    pub(crate) game: Game,
    pub(crate) selected_cell: Option<Position>,
    pub(crate) input_mode: InputMode,
    pub(crate) settings: Settings,
}

impl AppState {
    #[must_use]
    pub(crate) fn new(game: Game) -> Self {
        Self {
            game,
            selected_cell: None,
            input_mode: InputMode::Fill,
            settings: Settings::default(),
        }
    }

    #[must_use]
    pub(crate) fn new_with_settings_applied(game: Game) -> Self {
        let mut state = Self::new(game);
        state.apply_new_game_settings();
        state
    }

    pub(crate) fn apply_new_game_settings(&mut self) {
        if self.settings.assist.notes.auto_fill_notes_on_new_or_reset {
            self.game.auto_fill_notes_all_cells();
        }
    }

    pub(crate) fn reset_current_puzzle_state(&mut self) {
        for pos in Position::ALL {
            let _ = self.game.clear_cell(pos);
        }
        self.selected_cell = None;
        self.apply_new_game_settings();
    }

    #[must_use]
    pub(crate) fn rule_check_policy(&self) -> RuleCheckPolicy {
        if self.settings.assist.block_rule_violations {
            RuleCheckPolicy::Strict
        } else {
            RuleCheckPolicy::Permissive
        }
    }

    #[must_use]
    pub(crate) fn note_cleanup_policy(&self) -> NoteCleanupPolicy {
        if self.settings.assist.notes.auto_remove_peer_notes_on_fill {
            NoteCleanupPolicy::RemovePeers
        } else {
            NoteCleanupPolicy::None
        }
    }

    #[must_use]
    pub(crate) fn input_digit_options(&self) -> InputDigitOptions {
        InputDigitOptions::default()
            .rule_check_policy(self.rule_check_policy())
            .note_cleanup_policy(self.note_cleanup_policy())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub(crate) enum InputMode {
    Fill,
    Notes,
}

impl InputMode {
    pub(crate) fn toggle(&mut self) {
        *self = match self {
            InputMode::Fill => InputMode::Notes,
            InputMode::Notes => InputMode::Fill,
        }
    }

    #[must_use]
    pub(crate) fn swapped(self, swap: bool) -> Self {
        if swap {
            match self {
                InputMode::Fill => InputMode::Notes,
                InputMode::Notes => InputMode::Fill,
            }
        } else {
            self
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Settings {
    pub(crate) assist: AssistSettings,
}

#[derive(Debug, Clone)]
pub(crate) struct AssistSettings {
    pub(crate) block_rule_violations: bool,
    pub(crate) highlight: HighlightSettings,
    pub(crate) notes: NotesSettings,
}

impl Default for AssistSettings {
    fn default() -> Self {
        Self {
            block_rule_violations: true,
            highlight: HighlightSettings::default(),
            notes: NotesSettings::default(),
        }
    }
}

#[derive(Debug, Clone)]
#[expect(clippy::struct_excessive_bools)]
pub(crate) struct HighlightSettings {
    pub(crate) same_digit: bool,
    pub(crate) house_selected: bool,
    pub(crate) house_same_digit: bool,
    pub(crate) conflict: bool,
}

impl Default for HighlightSettings {
    fn default() -> Self {
        Self {
            same_digit: true,
            house_selected: true,
            house_same_digit: true,
            conflict: true,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NotesSettings {
    pub(crate) auto_remove_peer_notes_on_fill: bool,
    pub(crate) auto_fill_notes_on_new_or_reset: bool,
}

impl Default for NotesSettings {
    fn default() -> Self {
        Self {
            auto_remove_peer_notes_on_fill: true,
            auto_fill_notes_on_new_or_reset: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GhostType {
    Digit(Digit),
    Note(Digit),
}

#[derive(Debug, Clone)]
struct GameSnapshot {
    game: Game,
    selected_at_change: Option<Position>,
}

impl GameSnapshot {
    fn new(app_state: &AppState) -> Self {
        Self {
            game: app_state.game.clone(),
            selected_at_change: app_state.selected_cell,
        }
    }
}

#[derive(Debug, Clone)]
#[expect(dead_code)]
pub(crate) struct SolvabilityStats {
    pub(crate) assumptions_len: usize,
    pub(crate) backtrack_count: usize,
    pub(crate) solved_without_assumptions: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum SolvabilityState {
    Inconsistent,
    NoSolution,
    Solvable {
        with_user_notes: bool,
        stats: SolvabilityStats,
    },
}

#[derive(Debug)]
pub(crate) struct WorkEntry {
    pub(crate) handle: WorkHandle,
    pub(crate) responder: WorkResponder,
}

#[derive(Debug, Default)]
pub(crate) struct WorkState {
    pub(crate) in_flight: Vec<WorkEntry>,
    pub(crate) last_error: Option<WorkError>,
}

#[derive(Debug)]
pub(crate) struct UiState {
    pub(crate) active_modal: Option<ModalRequest>,
    pub(crate) conflict_ghost: Option<(Position, GhostType)>,
    pub(crate) work: WorkState,
    pub(crate) flow: FlowExecutor,
    history: UndoRedoStack<GameSnapshot>,
}

impl UiState {
    #[must_use]
    pub(crate) fn new(max_history_len: usize, init_state: &AppState) -> Self {
        let mut this = Self {
            active_modal: None,
            conflict_ghost: None,
            work: WorkState::default(),
            flow: FlowExecutor::new(),
            history: UndoRedoStack::new(max_history_len),
        };
        this.reset_history(init_state);
        this
    }

    pub(crate) fn reset_history(&mut self, init_state: &AppState) {
        self.history.clear();
        self.history.push(GameSnapshot::new(init_state));
    }

    #[must_use]
    pub(crate) fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub(crate) fn undo(&mut self, app_state: &mut AppState) -> bool {
        let Some(current) = self.history.current() else {
            return false;
        };
        let change_location = current.selected_at_change;
        if self.history.undo()
            && let Some(snapshot) = self.history.current()
        {
            app_state.game = snapshot.game.clone();
            app_state.selected_cell = change_location;
            true
        } else {
            false
        }
    }

    #[must_use]
    pub(crate) fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub(crate) fn redo(&mut self, app_state: &mut AppState) -> bool {
        if self.history.redo()
            && let Some(snapshot) = self.history.current()
        {
            app_state.game = snapshot.game.clone();
            app_state.selected_cell = snapshot.selected_at_change;
            true
        } else {
            false
        }
    }

    pub(crate) fn push_history(&mut self, app_state: &AppState) {
        self.history.push(GameSnapshot::new(app_state));
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{Digit, DigitGrid, Position};
    use numelace_game::{CellState, Game, InputDigitOptions};

    use super::{AppState, UiState};

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
    fn undo_redo_restores_game_and_selection() {
        let mut app_state = AppState::new(fixed_game());
        let mut ui_state = UiState::new(10, &app_state);

        app_state.selected_cell = Some(Position::new(0, 0));
        app_state
            .game
            .set_digit(
                Position::new(0, 0),
                Digit::D2,
                &InputDigitOptions::default(),
            )
            .unwrap();
        ui_state.push_history(&app_state);

        app_state.selected_cell = Some(Position::new(2, 0));
        app_state
            .game
            .set_digit(
                Position::new(2, 0),
                Digit::D3,
                &InputDigitOptions::default(),
            )
            .unwrap();
        ui_state.push_history(&app_state);

        assert!(ui_state.undo(&mut app_state));

        assert!(matches!(
            app_state.game.cell(Position::new(0, 0)),
            CellState::Filled(Digit::D2)
        ));
        assert!(matches!(
            app_state.game.cell(Position::new(2, 0)),
            CellState::Empty
        ));
        assert_eq!(app_state.selected_cell, Some(Position::new(2, 0)));

        assert!(ui_state.redo(&mut app_state));

        assert!(matches!(
            app_state.game.cell(Position::new(2, 0)),
            CellState::Filled(Digit::D3)
        ));
        assert_eq!(app_state.selected_cell, Some(Position::new(2, 0)));
    }
}
