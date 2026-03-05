use numelace_core::{Digit, Position};
use numelace_game::{Game, InputDigitOptions, NoteCleanupPolicy, RuleCheckPolicy};

use crate::state::{History, HistorySource, HistoryTarget, NewGameOptions, Settings};

// AppState holds persisted state (game/session + settings + history). It is serialized for resume.
#[derive(Debug)]
pub(crate) struct AppState {
    pub(crate) game: Game,
    selected_cell: Option<Position>,
    selected_digit: Option<Digit>,
    pub(crate) input_mode: InputMode,
    pub(crate) new_game_options: NewGameOptions,
    pub(crate) settings: Settings,
    history: History,
    dirty: bool,
}

impl AppState {
    #[must_use]
    pub(crate) fn new(game: Game) -> Self {
        let mut state = Self {
            game,
            selected_cell: None,
            selected_digit: None,
            input_mode: InputMode::Fill,
            new_game_options: NewGameOptions::default(),
            settings: Settings::default(),
            history: History::new(),
            dirty: false,
        };
        state.reset_history();
        state
    }

    #[must_use]
    pub(crate) fn new_with_settings_applied(game: Game) -> Self {
        let mut state = Self::new(game);
        state.apply_new_game_settings();
        state.reset_history();
        state
    }

    #[must_use]
    pub(crate) fn from_parts(
        game: Game,
        selected_cell: Option<Position>,
        mut selected_digit: Option<Digit>,
        input_mode: InputMode,
        new_game_options: NewGameOptions,
        settings: Settings,
        history: History,
    ) -> AppState {
        if selected_digit.is_none()
            && let Some(pos) = selected_cell
        {
            selected_digit = game.cell(pos).as_digit();
        }
        Self {
            game,
            selected_cell,
            selected_digit,
            input_mode,
            new_game_options,
            settings,
            history,
            dirty: false,
        }
    }

    pub(crate) fn access(&mut self) -> AppStateAccess<'_> {
        AppStateAccess { app_state: self }
    }

    #[must_use]
    pub(crate) fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    pub(crate) fn apply_new_game_settings(&mut self) {
        if self.settings.assist.notes.auto_fill_notes_on_new_or_reset {
            self.game.auto_fill_notes_all_cells();
        }
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

    #[must_use]
    pub(crate) fn selected_cell(&self) -> Option<Position> {
        self.selected_cell
    }

    #[must_use]
    pub(crate) fn selected_digit(&self) -> Option<Digit> {
        self.selected_digit
    }

    pub(crate) fn set_selected_cell(&mut self, pos: Position) {
        self.selected_cell = Some(pos);
        self.update_selected_digit();
    }

    pub(crate) fn update_selected_digit(&mut self) {
        if let Some(pos) = self.selected_cell
            && let Some(digit) = self.game.cell(pos).as_digit()
        {
            self.selected_digit = Some(digit);
        }
    }

    pub(crate) fn clear_selected_cell(&mut self) {
        self.selected_cell = None;
    }

    pub(crate) fn clear_selected_cell_and_digit(&mut self) {
        self.selected_cell = None;
        self.selected_digit = None;
    }

    #[must_use]
    pub(crate) fn history(&self) -> &History {
        &self.history
    }

    pub(crate) fn reset_history(&mut self) {
        self.history
            .reset(&HistorySource::new(&self.game, self.selected_cell));
    }

    #[must_use]
    pub(crate) fn build_undo_games(&self) -> Vec<Game> {
        self.history.build_undo_games(&self.game)
    }

    #[must_use]
    pub(crate) fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub(crate) fn undo(&mut self) -> bool {
        let mut selected_cell = self.selected_cell;
        if !self
            .history
            .undo(&mut HistoryTarget::new(&mut self.game, &mut selected_cell))
        {
            return false;
        }
        if let Some(pos) = selected_cell {
            self.set_selected_cell(pos);
        } else {
            self.clear_selected_cell_and_digit();
        }
        true
    }

    pub(crate) fn undo_steps(&mut self, steps: usize) -> bool {
        let mut selected_cell = self.selected_cell;
        if !self.history.undo_steps(
            steps,
            &mut HistoryTarget::new(&mut self.game, &mut selected_cell),
        ) {
            return false;
        }
        if let Some(pos) = selected_cell {
            self.set_selected_cell(pos);
        } else {
            self.clear_selected_cell_and_digit();
        }
        true
    }

    #[must_use]
    pub(crate) fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub(crate) fn redo(&mut self) -> bool {
        let mut selected_cell = self.selected_cell;
        if !self
            .history
            .redo(&mut HistoryTarget::new(&mut self.game, &mut selected_cell))
        {
            return false;
        }
        if let Some(pos) = selected_cell {
            self.set_selected_cell(pos);
        } else {
            self.clear_selected_cell_and_digit();
        }
        true
    }

    pub(crate) fn push_history(&mut self) {
        self.history
            .push(&HistorySource::new(&self.game, self.selected_cell));
    }
}

#[derive(Debug)]
pub(crate) struct AppStateAccess<'a> {
    app_state: &'a mut AppState,
}

impl AppStateAccess<'_> {
    #[must_use]
    pub(crate) fn as_ref(&self) -> &AppState {
        self.app_state
    }

    pub(crate) fn as_mut(&mut self) -> &mut AppState {
        self.app_state.dirty = true;
        self.app_state
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
