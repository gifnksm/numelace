use numelace_core::{Digit, Position};
use numelace_game::{Game, InputDigitOptions, NoteCleanupPolicy, RuleCheckPolicy};
use numelace_solver::BacktrackSolverStats;

use crate::history::UndoRedoStack;

#[derive(Debug)]
pub struct AppState {
    pub game: Game,
    pub selected_cell: Option<Position>,
    pub input_mode: InputMode,
    pub settings: Settings,
}

impl AppState {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            selected_cell: None,
            input_mode: InputMode::Fill,
            settings: Settings::default(),
        }
    }

    pub fn new_with_settings_applied(game: Game) -> Self {
        let mut state = Self::new(game);
        state.apply_new_game_settings();
        state
    }

    pub fn apply_new_game_settings(&mut self) {
        if self.settings.assist.notes.auto_fill_notes_on_new_or_reset {
            self.game.auto_fill_notes_all_cells();
        }
    }

    pub fn reset_current_puzzle_state(&mut self) {
        for pos in Position::ALL {
            let _ = self.game.clear_cell(pos);
        }
        self.selected_cell = None;
        self.apply_new_game_settings();
    }

    pub fn rule_check_policy(&self) -> RuleCheckPolicy {
        if self.settings.assist.block_rule_violations {
            RuleCheckPolicy::Strict
        } else {
            RuleCheckPolicy::Permissive
        }
    }

    pub fn note_cleanup_policy(&self) -> NoteCleanupPolicy {
        if self.settings.assist.notes.auto_remove_peer_notes_on_fill {
            NoteCleanupPolicy::RemovePeers
        } else {
            NoteCleanupPolicy::None
        }
    }

    pub fn input_digit_options(&self) -> InputDigitOptions {
        InputDigitOptions::default()
            .rule_check_policy(self.rule_check_policy())
            .note_cleanup_policy(self.note_cleanup_policy())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub enum InputMode {
    Fill,
    Notes,
}

impl InputMode {
    pub fn toggle(&mut self) {
        *self = match self {
            InputMode::Fill => InputMode::Notes,
            InputMode::Notes => InputMode::Fill,
        }
    }

    pub fn swapped(self, swap: bool) -> Self {
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
pub struct Settings {
    pub assist: AssistSettings,
}

#[derive(Debug, Clone)]
pub struct AssistSettings {
    pub block_rule_violations: bool,
    pub highlight: HighlightSettings,
    pub notes: NotesSettings,
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
pub struct HighlightSettings {
    pub same_digit: bool,
    pub house_selected: bool,
    pub house_same_digit: bool,
    pub conflict: bool,
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
pub struct NotesSettings {
    pub auto_remove_peer_notes_on_fill: bool,
    pub auto_fill_notes_on_new_or_reset: bool,
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
pub enum GhostType {
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
pub enum ModalKind {
    NewGameConfirm,
    ResetCurrentPuzzleConfirm,
    Settings,
    CheckSolvabilityResult(SolvabilityState),
}

#[derive(Debug, Clone)]
pub enum SolvabilityState {
    Inconsistent,
    NoSolution,
    Solvable {
        with_user_notes: bool,
        stats: BacktrackSolverStats,
    },
}

#[derive(Debug)]
pub struct UiState {
    pub active_modal: Option<ModalKind>,
    pub conflict_ghost: Option<(Position, GhostType)>,
    history: UndoRedoStack<GameSnapshot>,
}

impl UiState {
    pub fn new(max_history_len: usize, init_state: &AppState) -> Self {
        let mut this = Self {
            active_modal: None,
            conflict_ghost: None,
            history: UndoRedoStack::new(max_history_len),
        };
        this.reset_history(init_state);
        this
    }

    pub fn reset_history(&mut self, init_state: &AppState) {
        self.history.clear();
        self.history.push(GameSnapshot::new(init_state));
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn undo(&mut self, app_state: &mut AppState) -> bool {
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

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn redo(&mut self, app_state: &mut AppState) -> bool {
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

    pub fn push_history(&mut self, app_state: &AppState) {
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
