//! Game logic for Sudoku gameplay.
//!
//! This crate manages game sessions and player interactions, providing the core
//! game logic that sits between puzzle generation ([`numelace-generator`]) and the
//! user interface ([`numelace-app`]).
//!
//! # Overview
//!
//! The [`Game`] struct represents a single Sudoku game session, tracking:
//! - Initial puzzle cells (given/fixed cells that cannot be modified)
//! - Player-filled cells (editable by the player)
//! - Empty cells (not yet filled)
//!
//! # Design
//!
//! ## Design Decisions
//!
//! - **Permissive validation**: Allows rule-violating inputs (e.g., duplicate digits).
//!   Players can experiment freely, and mistakes are discovered organically.
//! - **Strict rule checks (optional)**: Operations accept a [`RuleCheckPolicy`]; `Strict`
//!   rejects inputs that would violate Sudoku rules, while `Permissive` allows them.
//! - **Completion detection**: A game is considered solved when all cells are filled
//!   and there are no rule violations (accepts any valid solution).
//! - **Cell state tracking**: Uses [`CellState`] enum to distinguish between given,
//!   filled, and empty cells at the type level.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use numelace_core::{Digit, Position};
//! use numelace_game::{CellState, Game, InputDigitOptions};
//! use numelace_generator::PuzzleGenerator;
//! use numelace_solver::TechniqueSolver;
//!
//! // Generate a puzzle
//! let solver = TechniqueSolver::with_all_techniques();
//! let generator = PuzzleGenerator::new(&solver);
//! let puzzle = generator.generate();
//!
//! // Start a new game
//! let mut game = Game::new(puzzle);
//!
//! // Find an empty cell
//! let empty_pos = *Position::ALL
//!     .iter()
//!     .find(|&&pos| game.cell(pos).is_empty())
//!     .expect("puzzle has empty cells");
//!
//! // Fill it with a digit
//! game.set_digit(empty_pos, Digit::D5, &InputDigitOptions::default())
//!     .unwrap();
//!
//! // Check if solved
//! if game.is_solved() {
//!     println!("Puzzle completed!");
//! }
//! ```

use numelace_core::{
    CandidateGrid, Digit, DigitGrid, DigitSet, Position,
    containers::{Array9, Array81},
    index::{DigitSemantics, PositionSemantics},
};
use numelace_generator::GeneratedPuzzle;

/// Errors that can occur during game operations.
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum GameError {
    /// Attempted to modify a given (initial) cell.
    ///
    /// Given cells are part of the initial puzzle and cannot be edited by the player.
    #[display("cannot modify a given cell")]
    CannotModifyGivenCell,
    /// Attempted to add notes to a filled cell.
    ///
    /// Notes can only be added to notes or empty cells.
    #[display("cannot add notes to a filled cell")]
    CannotAddNoteToFilledCell,
    /// Invalid notes data provided.
    ///
    /// The notes data must be a valid bitmask representing digits 1-9.
    #[display("invalid notes data: {_0:#x}")]
    InvalidNotes(#[error(not(source))] u16),
    /// Attempted to apply a digit that conflicts with existing digits.
    ///
    /// This occurs when the digit violates Sudoku rules in strict mode.
    #[display("given digit causes a conflict with existing digits")]
    ConflictingDigit,
}

/// Options that control digit input behavior.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct InputDigitOptions {
    rule_check_policy: RuleCheckPolicy,
    note_cleanup_policy: NoteCleanupPolicy,
}

impl InputDigitOptions {
    /// Creates options from explicit rule-check and note-cleanup policies.
    #[must_use]
    pub fn new(rule_check: RuleCheckPolicy, note_cleanup: NoteCleanupPolicy) -> Self {
        Self {
            rule_check_policy: rule_check,
            note_cleanup_policy: note_cleanup,
        }
    }

    /// Sets the rule check policy for this input.
    #[must_use]
    pub fn rule_check_policy(self, rule_check_policy: RuleCheckPolicy) -> Self {
        Self {
            rule_check_policy,
            ..self
        }
    }

    /// Sets the note cleanup policy for this input.
    #[must_use]
    pub fn note_cleanup_policy(self, note_cleanup_policy: NoteCleanupPolicy) -> Self {
        Self {
            note_cleanup_policy,
            ..self
        }
    }
}

/// Controls whether rule-violating inputs are permitted.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub enum RuleCheckPolicy {
    /// Allow inputs even if they conflict with existing digits.
    #[default]
    Permissive,
    /// Reject inputs that conflict with existing digits.
    Strict,
}

/// Controls how notes are cleaned up after digit entry.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub enum NoteCleanupPolicy {
    /// Do not modify notes in peer cells.
    #[default]
    None,
    /// Remove the placed digit from notes in peer cells.
    RemovePeers,
}

/// Indicates whether an input is currently allowed and, if not, why it is blocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub enum InputCapability {
    /// The input is allowed.
    Allowed,
    /// The target cell is a given cell and cannot be modified.
    BlockedByGivenCell,
    /// The target cell is filled and cannot accept notes.
    BlockedByFilledCell,
    /// The input conflicts with an existing digit under strict rules.
    BlockedByConflict,
}

/// A Sudoku game session.
///
/// Manages the game state, including given (initial) cells and player input.
/// Provides operations for filling and clearing cells, with validation to prevent
/// modification of given cells.
///
/// # Example
///
/// ```
/// use numelace_game::Game;
/// use numelace_generator::PuzzleGenerator;
/// use numelace_solver::TechniqueSolver;
///
/// let solver = TechniqueSolver::with_all_techniques();
/// let generator = PuzzleGenerator::new(&solver);
/// let puzzle = generator.generate();
/// let game = Game::new(puzzle);
///
/// // Game tracks given cells and player input separately
/// assert!(!game.is_solved()); // Newly created game is not solved
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    grid: Array81<CellState, PositionSemantics>,
}

impl Game {
    /// Creates a new game from a generated puzzle.
    ///
    /// All cells from the puzzle's problem grid are marked as given (fixed) cells.
    /// Empty cells in the problem are left as [`CellState::Empty`].
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_game::Game;
    /// use numelace_generator::PuzzleGenerator;
    /// use numelace_solver::TechniqueSolver;
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let generator = PuzzleGenerator::new(&solver);
    /// let puzzle = generator.generate();
    /// let game = Game::new(puzzle);
    /// ```
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(puzzle: GeneratedPuzzle) -> Self {
        let mut grid = Array81::from_array([const { CellState::Empty }; 81]);
        for pos in Position::ALL {
            if let Some(digit) = puzzle.problem[pos] {
                grid[pos] = CellState::Given(digit);
            }
        }
        Self { grid }
    }

    /// Creates a game from a problem grid and a filled (player input) grid.
    ///
    /// Cells with digits in `problem` are treated as givens. Digits in `filled`
    /// are applied as player-entered values.
    ///
    /// # Errors
    ///
    /// Returns [`GameError::CannotModifyGivenCell`] if `filled` contains a digit
    /// in a position that is a given in `problem`.
    pub fn from_problem_filled_notes(
        problem: &DigitGrid,
        filled: &DigitGrid,
        notes: &[[u16; 9]; 9],
    ) -> Result<Self, GameError> {
        let mut grid = Array81::from_array([const { CellState::Empty }; 81]);
        for pos in Position::ALL {
            if let Some(digit) = problem[pos] {
                grid[pos] = CellState::Given(digit);
            }
        }

        let mut this = Self { grid };
        for pos in Position::ALL {
            if let Some(digit) = filled[pos] {
                this.set_digit(pos, digit, &InputDigitOptions::default())?;
            }
        }

        for (y, row) in (0..9).zip(notes) {
            for (x, bits) in (0..9).zip(row) {
                let pos = Position::new(x, y);
                let digits =
                    DigitSet::try_from_bits(*bits).ok_or(GameError::InvalidNotes(*bits))?;
                for d in digits {
                    this.toggle_note(pos, d, RuleCheckPolicy::Permissive)?;
                }
            }
        }

        Ok(this)
    }

    /// Returns the state of the cell at the given position.
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_core::Position;
    /// use numelace_game::{CellState, Game};
    /// use numelace_generator::PuzzleGenerator;
    /// use numelace_solver::TechniqueSolver;
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let generator = PuzzleGenerator::new(&solver);
    /// let puzzle = generator.generate();
    /// let game = Game::new(puzzle);
    ///
    /// let pos = Position::new(0, 0);
    /// match game.cell(pos) {
    ///     CellState::Given(digit) => println!("Given cell: {}", digit),
    ///     CellState::Filled(digit) => println!("Player filled: {}", digit),
    ///     CellState::Notes(digits) => println!("Notes: {:?}", digits),
    ///     CellState::Empty => println!("Empty cell"),
    /// }
    /// ```
    #[must_use]
    pub fn cell(&self, pos: Position) -> &CellState {
        &self.grid[pos]
    }

    /// Checks if the game is solved.
    ///
    /// A game is considered solved when:
    /// - All cells are filled (no empty cells)
    /// - There are no rule violations (no duplicate digits in rows, columns, or boxes)
    ///
    /// This accepts any valid solution, not just the original solution from the generator.
    /// This handles puzzles with multiple solutions correctly.
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_core::{Digit, Position};
    /// use numelace_game::{Game, InputDigitOptions};
    /// use numelace_generator::PuzzleGenerator;
    /// use numelace_solver::TechniqueSolver;
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let generator = PuzzleGenerator::new(&solver);
    /// let puzzle = generator.generate();
    /// let mut game = Game::new(puzzle.clone());
    ///
    /// // Fill all empty cells with the solution
    /// for pos in Position::ALL {
    ///     if game.cell(pos).is_empty() {
    ///         let digit = puzzle.solution[pos].unwrap();
    ///         game.set_digit(pos, digit, &InputDigitOptions::default())
    ///             .unwrap();
    ///     }
    /// }
    ///
    /// assert!(game.is_solved());
    /// ```
    #[must_use]
    pub fn is_solved(&self) -> bool {
        let grid = self.to_candidate_grid();
        grid.is_solved().unwrap_or_default()
    }

    #[must_use]
    fn to_candidate_grid(&self) -> CandidateGrid {
        let mut candidate_grid = CandidateGrid::new();
        for pos in Position::ALL {
            match &self.grid[pos] {
                CellState::Given(digit) | CellState::Filled(digit) => {
                    candidate_grid.place(pos, *digit);
                }
                CellState::Notes(_) | CellState::Empty => {}
            }
        }
        candidate_grid
    }

    fn is_conflicting(&self, pos: Position, digit: Digit) -> bool {
        for peer_pos in pos.house_peers() {
            if self.grid[peer_pos].as_digit() == Some(digit) {
                return true;
            }
        }
        false
    }

    /// Places a digit at the given position.
    ///
    /// If the cell is empty, it becomes filled. If the cell is already filled,
    /// the digit is replaced.
    ///
    /// # Errors
    ///
    /// Returns [`GameError::CannotModifyGivenCell`] if the position contains a given cell.
    /// Returns [`GameError::ConflictingDigit`] if strict rule checks are enabled and
    /// the digit conflicts with existing digits.
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_core::{Digit, Position};
    /// use numelace_game::{Game, InputDigitOptions};
    /// use numelace_generator::PuzzleGenerator;
    /// use numelace_solver::TechniqueSolver;
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let generator = PuzzleGenerator::new(&solver);
    /// let puzzle = generator.generate();
    /// let mut game = Game::new(puzzle);
    ///
    /// // Find an empty cell
    /// let empty_pos = *Position::ALL
    ///     .iter()
    ///     .find(|&&pos| game.cell(pos).is_empty())
    ///     .expect("puzzle has empty cells");
    ///
    /// // Fill it
    /// game.set_digit(empty_pos, Digit::D5, &InputDigitOptions::default())
    ///     .unwrap();
    /// assert_eq!(game.cell(empty_pos).as_digit(), Some(Digit::D5));
    /// ```
    pub fn set_digit(
        &mut self,
        pos: Position,
        digit: Digit,
        options: &InputDigitOptions,
    ) -> Result<(), GameError> {
        if self.grid[pos].as_filled() == Some(digit) {
            return Ok(());
        }

        match options.rule_check_policy {
            RuleCheckPolicy::Strict => {
                if self.is_conflicting(pos, digit) {
                    return Err(GameError::ConflictingDigit);
                }
            }
            RuleCheckPolicy::Permissive => {}
        }

        self.grid[pos].set_filled(digit)?;

        match options.note_cleanup_policy {
            NoteCleanupPolicy::None => {}
            NoteCleanupPolicy::RemovePeers => {
                for pos in pos.house_peers() {
                    self.grid[pos].drop_note_digit(digit);
                }
            }
        }

        Ok(())
    }

    /// Returns the capability for placing a digit at the given position.
    ///
    /// The returned [`InputCapability`] indicates if the input is allowed or the
    /// reason it is blocked, taking the provided policy into account.
    /// Setting the same digit again is always allowed, even under strict checks.
    #[must_use]
    pub fn set_digit_capability(
        &self,
        pos: Position,
        digit: Digit,
        policy: RuleCheckPolicy,
    ) -> InputCapability {
        if self.cell(pos).is_given() {
            return InputCapability::BlockedByGivenCell;
        }
        if self.cell(pos).as_filled() == Some(digit) {
            // Setting the same digit again is always allowed
            return InputCapability::Allowed;
        }
        match policy {
            RuleCheckPolicy::Strict => {
                for peer_pos in pos.house_peers() {
                    if self.grid[peer_pos].as_digit() == Some(digit) {
                        return InputCapability::BlockedByConflict;
                    }
                }
            }
            RuleCheckPolicy::Permissive => {}
        }
        InputCapability::Allowed
    }

    /// Toggles a candidate note at the given position.
    ///
    /// If the cell is empty, it becomes a notes cell with the digit. If the cell already
    /// has notes, the digit is toggled; when the last note is removed, the cell becomes empty.
    ///
    /// # Errors
    ///
    /// Returns [`GameError::CannotModifyGivenCell`] if the position contains a given cell.
    /// Returns [`GameError::CannotAddNoteToFilledCell`] if the position contains a filled cell.
    /// Returns [`GameError::ConflictingDigit`] if strict rule checks are enabled and
    /// the digit conflicts with existing digits.
    /// Note removal is always allowed even under strict rule checks.
    pub fn toggle_note(
        &mut self,
        pos: Position,
        digit: Digit,
        policy: RuleCheckPolicy,
    ) -> Result<(), GameError> {
        let removing_note = self.grid[pos]
            .as_notes()
            .is_some_and(|notes| notes.contains(digit));
        if !removing_note {
            match policy {
                RuleCheckPolicy::Strict => {
                    if self.is_conflicting(pos, digit) {
                        return Err(GameError::ConflictingDigit);
                    }
                }
                RuleCheckPolicy::Permissive => {}
            }
        }
        self.grid[pos].toggle_note(digit)?;
        Ok(())
    }

    /// Returns the toggle capability for notes at the given position.
    ///
    /// The returned [`InputCapability`] indicates if the input is allowed or the
    /// reason it is blocked, taking the provided policy into account.
    /// Note removal returns [`InputCapability::Allowed`] even under strict checks.
    #[must_use]
    pub fn toggle_note_capability(
        &self,
        pos: Position,
        digit: Digit,
        policy: RuleCheckPolicy,
    ) -> InputCapability {
        match self.cell(pos) {
            CellState::Given(_) => return InputCapability::BlockedByGivenCell,
            CellState::Filled(_) => return InputCapability::BlockedByFilledCell,
            CellState::Notes(_) | CellState::Empty => {}
        }
        if self
            .cell(pos)
            .as_notes()
            .is_some_and(|notes| notes.contains(digit))
        {
            // Clearing is always allowed
            return InputCapability::Allowed;
        }
        match policy {
            RuleCheckPolicy::Strict => {
                if self.is_conflicting(pos, digit) {
                    return InputCapability::BlockedByConflict;
                }
            }
            RuleCheckPolicy::Permissive => {}
        }
        InputCapability::Allowed
    }

    /// Clears the digit at the given position.
    ///
    /// If the cell is filled, it becomes empty. If the cell is already empty,
    /// this operation has no effect.
    ///
    /// # Errors
    ///
    /// Returns [`GameError::CannotModifyGivenCell`] if the position contains a given cell.
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_core::{Digit, Position};
    /// use numelace_game::{Game, InputDigitOptions};
    /// use numelace_generator::PuzzleGenerator;
    /// use numelace_solver::TechniqueSolver;
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let generator = PuzzleGenerator::new(&solver);
    /// let puzzle = generator.generate();
    /// let mut game = Game::new(puzzle);
    ///
    /// // Find an empty cell and fill it
    /// let empty_pos = *Position::ALL
    ///     .iter()
    ///     .find(|&&pos| game.cell(pos).is_empty())
    ///     .expect("puzzle has empty cells");
    /// game.set_digit(empty_pos, Digit::D5, &InputDigitOptions::default())
    ///     .unwrap();
    ///
    /// // Clear it
    /// game.clear_cell(empty_pos).unwrap();
    /// assert!(game.cell(empty_pos).is_empty());
    /// ```
    pub fn clear_cell(&mut self, pos: Position) -> Result<(), GameError> {
        self.grid[pos].clear()?;
        Ok(())
    }

    /// Returns whether the cell currently contains a removable digit.
    ///
    /// This is `true` only for filled (player-entered) cells.
    #[must_use]
    pub fn has_removable_digit(&self, pos: Position) -> bool {
        self.cell(pos).is_filled()
    }

    /// Returns the count of each decided digit (given or filled) on the board.
    ///
    /// The returned array is indexed by [`Digit`] and includes both given and
    /// player-filled cells.
    #[must_use]
    pub fn decided_digit_count(&self) -> Array9<usize, DigitSemantics> {
        let mut counts = Array9::from_array([0; 9]);
        for pos in Position::ALL {
            if let Some(digit) = self.cell(pos).as_digit() {
                counts[digit] += 1;
            }
        }
        counts
    }
}

/// The state of a cell in the game.
///
/// This enum distinguishes between four types of cells:
/// - [`Given`]: Initial puzzle cells (cannot be modified)
/// - [`Filled`]: Player-filled cells (can be modified or cleared)
/// - [`Notes`]: Candidate notes (player memo)
/// - [`Empty`]: Cells that have not been filled yet
///
/// [`Given`]: CellState::Given
/// [`Filled`]: CellState::Filled
/// [`Notes`]: CellState::Notes
/// [`Empty`]: CellState::Empty
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub enum CellState {
    /// A cell from the initial puzzle (cannot be modified by the player).
    Given(Digit),
    /// A cell filled by the player (can be modified or cleared).
    Filled(Digit),
    /// Candidate notes for this cell.
    Notes(DigitSet),
    /// An empty cell (not yet filled).
    Empty,
}

impl CellState {
    /// Sets this cell to a filled digit, clearing notes if needed.
    ///
    /// # Errors
    ///
    /// Returns [`GameError::CannotModifyGivenCell`] if this is a given cell.
    pub fn set_filled(&mut self, digit: Digit) -> Result<(), GameError> {
        match self {
            CellState::Given(_) => Err(GameError::CannotModifyGivenCell),
            CellState::Filled(d) => {
                *d = digit;
                Ok(())
            }
            CellState::Notes(_) | CellState::Empty => {
                *self = CellState::Filled(digit);
                Ok(())
            }
        }
    }

    /// Drops a digit from this cell's notes, converting empty notes to [`CellState::Empty`].
    pub fn drop_note_digit(&mut self, digit: Digit) {
        if let CellState::Notes(notes) = self {
            notes.remove(digit);
            if notes.is_empty() {
                *self = CellState::Empty;
            }
        }
    }

    /// Toggles a note in this cell.
    ///
    /// # Errors
    ///
    /// Returns [`GameError::CannotModifyGivenCell`] if this is a given cell.
    /// Returns [`GameError::CannotAddNoteToFilledCell`] if this is a filled cell.
    pub fn toggle_note(&mut self, digit: Digit) -> Result<(), GameError> {
        match self {
            CellState::Given(_) => Err(GameError::CannotModifyGivenCell),
            CellState::Filled(_) => Err(GameError::CannotAddNoteToFilledCell),
            CellState::Notes(digits) => {
                digits.toggle(digit);
                if digits.is_empty() {
                    *self = CellState::Empty;
                }
                Ok(())
            }
            CellState::Empty => {
                let mut digits = DigitSet::new();
                digits.insert(digit);
                *self = CellState::Notes(digits);
                Ok(())
            }
        }
    }

    /// Clears this cell if it contains player input or notes.
    ///
    /// # Errors
    ///
    /// Returns [`GameError::CannotModifyGivenCell`] if this is a given cell.
    pub fn clear(&mut self) -> Result<(), GameError> {
        match self {
            CellState::Given(_) => Err(GameError::CannotModifyGivenCell),
            CellState::Filled(_) | CellState::Notes(_) => {
                *self = CellState::Empty;
                Ok(())
            }
            CellState::Empty => Ok(()),
        }
    }

    /// Returns the digit if this is a given cell, otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_core::Digit;
    /// use numelace_game::CellState;
    ///
    /// assert_eq!(CellState::Given(Digit::D5).as_given(), Some(Digit::D5));
    /// assert_eq!(CellState::Filled(Digit::D5).as_given(), None);
    /// assert_eq!(CellState::Empty.as_given(), None);
    /// ```
    #[must_use]
    pub fn as_given(&self) -> Option<Digit> {
        match self {
            CellState::Given(digit) => Some(*digit),
            _ => None,
        }
    }

    /// Returns the digit if this is a filled cell, otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_core::Digit;
    /// use numelace_game::CellState;
    ///
    /// assert_eq!(CellState::Filled(Digit::D5).as_filled(), Some(Digit::D5));
    /// assert_eq!(CellState::Given(Digit::D5).as_filled(), None);
    /// assert_eq!(CellState::Empty.as_filled(), None);
    /// ```
    #[must_use]
    pub fn as_filled(&self) -> Option<Digit> {
        match self {
            CellState::Filled(digit) => Some(*digit),
            _ => None,
        }
    }

    /// Returns the notes if this is a notes cell, otherwise `None`.
    #[must_use]
    pub fn as_notes(&self) -> Option<DigitSet> {
        match self {
            CellState::Notes(digits) => Some(*digits),
            _ => None,
        }
    }

    /// Returns the digit if this cell contains one (given or filled), otherwise `None`.
    ///
    /// This is a convenience method that returns the digit regardless of whether
    /// it's a given or filled cell.
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_core::Digit;
    /// use numelace_game::CellState;
    ///
    /// assert_eq!(CellState::Given(Digit::D5).as_digit(), Some(Digit::D5));
    /// assert_eq!(CellState::Filled(Digit::D7).as_digit(), Some(Digit::D7));
    /// assert_eq!(CellState::Empty.as_digit(), None);
    /// ```
    #[must_use]
    pub fn as_digit(&self) -> Option<Digit> {
        match self {
            CellState::Given(digit) | CellState::Filled(digit) => Some(*digit),
            CellState::Notes(_) | CellState::Empty => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{Digit, DigitGrid, DigitSet, Position};
    use numelace_generator::PuzzleGenerator;

    use super::*;

    #[test]
    fn test_new_game_preserves_puzzle_structure() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let game = Game::new(puzzle.clone());

        // Given cells match problem
        for pos in Position::ALL {
            match puzzle.problem[pos] {
                Some(digit) => {
                    assert_eq!(game.cell(pos), &CellState::Given(digit));
                }
                None => {
                    assert_eq!(game.cell(pos), &CellState::Empty);
                }
            }
        }
    }

    #[test]
    fn test_from_problem_filled() {
        let problem: DigitGrid = format!("1{}", ".".repeat(80))
            .parse()
            .expect("valid problem grid");
        let filled: DigitGrid = format!(".2{}", ".".repeat(79))
            .parse()
            .expect("valid filled grid");

        let game = Game::from_problem_filled_notes(&problem, &filled, &[[0; 9]; 9])
            .expect("compatible grids");

        assert_eq!(game.cell(Position::new(0, 0)), &CellState::Given(Digit::D1));
        assert_eq!(
            game.cell(Position::new(1, 0)),
            &CellState::Filled(Digit::D2)
        );

        let conflict: DigitGrid = format!("3{}", ".".repeat(80))
            .parse()
            .expect("valid filled grid");
        assert!(matches!(
            Game::from_problem_filled_notes(&problem, &conflict, &[[0; 9]; 9]),
            Err(GameError::CannotModifyGivenCell)
        ));
    }

    #[test]
    fn test_set_digit_basic_operations() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        let empty_pos = *Position::ALL
            .iter()
            .find(|&&pos| game.cell(pos).is_empty())
            .expect("puzzle has empty cells");

        // Can fill empty cell
        assert!(
            game.set_digit(empty_pos, Digit::D5, &InputDigitOptions::default())
                .is_ok()
        );
        assert_eq!(game.cell(empty_pos), &CellState::Filled(Digit::D5));

        // Can replace filled cell
        assert!(
            game.set_digit(empty_pos, Digit::D7, &InputDigitOptions::default())
                .is_ok()
        );
        assert_eq!(game.cell(empty_pos), &CellState::Filled(Digit::D7));

        // Re-entering the same digit is a no-op
        assert!(
            game.set_digit(empty_pos, Digit::D7, &InputDigitOptions::default())
                .is_ok()
        );
        assert_eq!(game.cell(empty_pos), &CellState::Filled(Digit::D7));
    }

    #[test]
    fn test_set_digit_note_cleanup_removes_peer_notes() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        let empty_pos = *Position::ALL
            .iter()
            .find(|&&pos| game.cell(pos).is_empty())
            .expect("puzzle has empty cells");
        let peer_pos = empty_pos
            .house_peers()
            .into_iter()
            .find(|pos| game.cell(*pos).is_empty())
            .expect("house has an empty peer");

        game.toggle_note(peer_pos, Digit::D5, RuleCheckPolicy::Permissive)
            .unwrap();
        assert!(matches!(
            game.cell(peer_pos),
            CellState::Notes(notes) if notes.contains(Digit::D5)
        ));

        game.set_digit(
            empty_pos,
            Digit::D5,
            &InputDigitOptions::default().note_cleanup_policy(NoteCleanupPolicy::RemovePeers),
        )
        .unwrap();

        assert!(matches!(game.cell(peer_pos), CellState::Empty));
    }

    #[test]
    fn test_set_digit_note_cleanup_none_keeps_peer_notes() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        let empty_pos = *Position::ALL
            .iter()
            .find(|&&pos| game.cell(pos).is_empty())
            .expect("puzzle has empty cells");
        let peer_pos = empty_pos
            .house_peers()
            .into_iter()
            .find(|pos| game.cell(*pos).is_empty())
            .expect("house has an empty peer");

        game.toggle_note(peer_pos, Digit::D4, RuleCheckPolicy::Permissive)
            .unwrap();
        assert!(matches!(
            game.cell(peer_pos),
            CellState::Notes(notes) if notes.contains(Digit::D4)
        ));

        game.set_digit(empty_pos, Digit::D4, &InputDigitOptions::default())
            .unwrap();

        assert!(matches!(
            game.cell(peer_pos),
            CellState::Notes(notes) if notes.contains(Digit::D4)
        ));
    }

    #[test]
    fn test_set_digit_strict_conflict_does_not_cleanup_peer_notes() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        let empty_pos = *Position::ALL
            .iter()
            .find(|&&pos| game.cell(pos).is_empty())
            .expect("puzzle has empty cells");
        let mut peer_positions = empty_pos
            .house_peers()
            .into_iter()
            .filter(|pos| game.cell(*pos).is_empty());
        let conflict_pos = peer_positions
            .next()
            .expect("house has an empty peer for conflict");
        let note_pos = peer_positions
            .next()
            .expect("house has an empty peer for notes");

        game.toggle_note(note_pos, Digit::D5, RuleCheckPolicy::Permissive)
            .unwrap();
        assert!(matches!(
            game.cell(note_pos),
            CellState::Notes(notes) if notes.contains(Digit::D5)
        ));

        game.set_digit(conflict_pos, Digit::D5, &InputDigitOptions::default())
            .unwrap();

        let result = game.set_digit(
            empty_pos,
            Digit::D5,
            &InputDigitOptions::default()
                .rule_check_policy(RuleCheckPolicy::Strict)
                .note_cleanup_policy(NoteCleanupPolicy::RemovePeers),
        );
        assert!(matches!(result, Err(GameError::ConflictingDigit)));

        assert!(matches!(
            game.cell(note_pos),
            CellState::Notes(notes) if notes.contains(Digit::D5)
        ));
    }

    #[test]
    fn test_toggle_note_basic_operations() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        let given_pos = *Position::ALL
            .iter()
            .find(|&&pos| game.cell(pos).is_given())
            .expect("puzzle has given cells");
        let empty_pos = *Position::ALL
            .iter()
            .find(|&&pos| game.cell(pos).is_empty())
            .expect("puzzle has empty cells");

        assert!(
            game.toggle_note(empty_pos, Digit::D4, RuleCheckPolicy::Permissive)
                .is_ok()
        );
        assert!(matches!(
            game.cell(empty_pos),
            CellState::Notes(digits) if digits.contains(Digit::D4)
        ));

        assert!(
            game.toggle_note(empty_pos, Digit::D4, RuleCheckPolicy::Permissive)
                .is_ok()
        );
        assert!(game.cell(empty_pos).is_empty());

        game.set_digit(empty_pos, Digit::D7, &InputDigitOptions::default())
            .unwrap();
        assert!(matches!(
            game.toggle_note(empty_pos, Digit::D3, RuleCheckPolicy::Permissive),
            Err(GameError::CannotAddNoteToFilledCell)
        ));

        assert!(matches!(
            game.toggle_note(given_pos, Digit::D3, RuleCheckPolicy::Permissive),
            Err(GameError::CannotModifyGivenCell)
        ));
    }

    #[test]
    fn test_strict_conflict_rejects_inputs() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        let empty_pos = *Position::ALL
            .iter()
            .find(|&&pos| game.cell(pos).is_empty())
            .expect("puzzle has empty cells");
        let peer_pos = empty_pos
            .house_peers()
            .into_iter()
            .find(|pos| game.cell(*pos).is_empty())
            .expect("house has an empty peer");

        game.set_digit(peer_pos, Digit::D5, &InputDigitOptions::default())
            .unwrap();

        assert!(matches!(
            game.set_digit(
                empty_pos,
                Digit::D5,
                &InputDigitOptions::default().rule_check_policy(RuleCheckPolicy::Strict)
            ),
            Err(GameError::ConflictingDigit)
        ));
        assert!(matches!(
            game.toggle_note(empty_pos, Digit::D5, RuleCheckPolicy::Strict),
            Err(GameError::ConflictingDigit)
        ));

        game.set_digit(empty_pos, Digit::D5, &InputDigitOptions::default())
            .unwrap();
        assert_eq!(
            game.set_digit_capability(empty_pos, Digit::D5, RuleCheckPolicy::Strict),
            InputCapability::Allowed
        );
        assert!(matches!(game.clear_cell(empty_pos), Ok(())));
        assert!(game.cell(empty_pos).is_empty());

        game.toggle_note(empty_pos, Digit::D5, RuleCheckPolicy::Permissive)
            .unwrap();
        assert_eq!(
            game.toggle_note_capability(empty_pos, Digit::D5, RuleCheckPolicy::Strict),
            InputCapability::Allowed
        );
        assert!(matches!(
            game.toggle_note(empty_pos, Digit::D5, RuleCheckPolicy::Strict),
            Ok(())
        ));
        assert!(game.cell(empty_pos).is_empty());

        assert_eq!(
            game.set_digit_capability(empty_pos, Digit::D5, RuleCheckPolicy::Strict),
            InputCapability::BlockedByConflict
        );
        assert_eq!(
            game.toggle_note_capability(empty_pos, Digit::D5, RuleCheckPolicy::Strict),
            InputCapability::BlockedByConflict
        );
        assert_eq!(
            game.set_digit_capability(empty_pos, Digit::D5, RuleCheckPolicy::Permissive),
            InputCapability::Allowed
        );
    }

    #[test]
    fn test_cannot_modify_given_cells() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        let given_pos = *Position::ALL
            .iter()
            .find(|&&pos| game.cell(pos).is_given())
            .expect("puzzle has given cells");

        // Cannot set digit on given cell
        assert!(matches!(
            game.set_digit(given_pos, Digit::D5, &InputDigitOptions::default()),
            Err(GameError::CannotModifyGivenCell)
        ));

        // Cannot clear given cell
        assert!(matches!(
            game.clear_cell(given_pos),
            Err(GameError::CannotModifyGivenCell)
        ));
    }

    #[test]
    fn test_clear_cell_operations() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        let empty_pos = *Position::ALL
            .iter()
            .find(|&&pos| game.cell(pos).is_empty())
            .expect("puzzle has empty cells");

        // Fill then clear
        game.set_digit(empty_pos, Digit::D5, &InputDigitOptions::default())
            .unwrap();
        assert!(game.cell(empty_pos).is_filled());

        game.clear_cell(empty_pos).unwrap();
        assert!(game.cell(empty_pos).is_empty());

        // Clear empty cell is no-op
        assert!(game.clear_cell(empty_pos).is_ok());
        assert!(game.cell(empty_pos).is_empty());
    }

    #[test]
    fn test_digit_capability_helpers() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        let given_pos = Position::ALL
            .into_iter()
            .find(|&pos| game.cell(pos).is_given())
            .expect("puzzle has given cells");
        let empty_pos = Position::ALL
            .into_iter()
            .find(|&pos| game.cell(pos).is_empty())
            .expect("puzzle has empty cells");

        assert_eq!(
            game.set_digit_capability(given_pos, Digit::D1, RuleCheckPolicy::Permissive),
            InputCapability::BlockedByGivenCell
        );
        assert_eq!(
            game.toggle_note_capability(given_pos, Digit::D1, RuleCheckPolicy::Permissive),
            InputCapability::BlockedByGivenCell
        );
        assert_eq!(
            game.set_digit_capability(empty_pos, Digit::D1, RuleCheckPolicy::Permissive),
            InputCapability::Allowed
        );
        assert_eq!(
            game.toggle_note_capability(empty_pos, Digit::D1, RuleCheckPolicy::Permissive),
            InputCapability::Allowed
        );

        assert!(!game.has_removable_digit(empty_pos));
        game.set_digit(empty_pos, Digit::D5, &InputDigitOptions::default())
            .unwrap();
        assert!(game.has_removable_digit(empty_pos));
        assert_eq!(
            game.toggle_note_capability(empty_pos, Digit::D1, RuleCheckPolicy::Permissive),
            InputCapability::BlockedByFilledCell
        );
    }

    #[test]
    fn test_decided_digit_count_counts_given_and_filled() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        let empty_positions: Vec<Position> = Position::ALL
            .iter()
            .copied()
            .filter(|&pos| game.cell(pos).is_empty())
            .collect();

        let first = empty_positions
            .first()
            .copied()
            .expect("puzzle has empty cells");
        let second = empty_positions
            .get(1)
            .copied()
            .expect("puzzle has at least two empty cells");

        let d5_before = game.decided_digit_count()[Digit::D5];
        game.set_digit(first, Digit::D5, &InputDigitOptions::default())
            .unwrap();
        game.set_digit(second, Digit::D5, &InputDigitOptions::default())
            .unwrap();

        let counts = game.decided_digit_count();
        assert_eq!(counts[Digit::D5], d5_before + 2);
    }

    #[test]
    fn test_is_solved_with_complete_solution() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle.clone());

        // Initially not solved
        assert!(!game.is_solved());

        // Fill all empty cells with solution
        for pos in Position::ALL {
            if game.cell(pos).is_empty() {
                let digit = puzzle.solution[pos].expect("solution is complete");
                game.set_digit(pos, digit, &InputDigitOptions::default())
                    .unwrap();
            }
        }

        // Now solved
        assert!(game.is_solved());
    }

    #[test]
    fn test_is_solved_with_conflicts() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        // Fill all cells with D1 (creates conflicts)
        for pos in Position::ALL {
            if game.cell(pos).is_empty() {
                let _ = game.set_digit(pos, Digit::D1, &InputDigitOptions::default());
            }
        }

        // Not solved due to conflicts
        assert!(!game.is_solved());
    }

    #[test]
    fn test_cell_state_helpers() {
        // as_given
        assert_eq!(CellState::Given(Digit::D5).as_given(), Some(Digit::D5));
        assert_eq!(CellState::Filled(Digit::D5).as_given(), None);
        assert_eq!(CellState::Empty.as_given(), None);

        // as_filled
        assert_eq!(CellState::Filled(Digit::D5).as_filled(), Some(Digit::D5));
        assert_eq!(CellState::Given(Digit::D5).as_filled(), None);
        assert_eq!(CellState::Empty.as_filled(), None);

        // as_notes
        let mut notes = DigitSet::new();
        notes.insert(Digit::D3);
        assert_eq!(CellState::Notes(notes).as_notes(), Some(notes));
        assert_eq!(CellState::Empty.as_notes(), None);

        // as_digit (unified access)
        assert_eq!(CellState::Given(Digit::D5).as_digit(), Some(Digit::D5));
        assert_eq!(CellState::Filled(Digit::D7).as_digit(), Some(Digit::D7));
        assert_eq!(CellState::Notes(notes).as_digit(), None);
        assert_eq!(CellState::Empty.as_digit(), None);
    }

    #[test]
    fn test_cell_state_set_filled_transitions() {
        let mut cell = CellState::Empty;
        cell.set_filled(Digit::D4).unwrap();
        assert_eq!(cell, CellState::Filled(Digit::D4));

        let mut cell = CellState::Notes({
            let mut notes = DigitSet::new();
            notes.insert(Digit::D2);
            notes
        });
        cell.set_filled(Digit::D8).unwrap();
        assert_eq!(cell, CellState::Filled(Digit::D8));
    }

    #[test]
    fn test_cell_state_drop_note_digit_clears_empty() {
        let mut notes = DigitSet::new();
        notes.insert(Digit::D6);
        let mut cell = CellState::Notes(notes);
        cell.drop_note_digit(Digit::D6);
        assert_eq!(cell, CellState::Empty);
    }

    #[test]
    fn test_cell_state_toggle_note_transitions() {
        let mut cell = CellState::Empty;
        cell.toggle_note(Digit::D3).unwrap();
        assert!(matches!(
            cell,
            CellState::Notes(notes) if notes.contains(Digit::D3)
        ));

        cell.toggle_note(Digit::D3).unwrap();
        assert_eq!(cell, CellState::Empty);
    }

    #[test]
    fn test_cell_state_clear_transitions() {
        let mut cell = CellState::Filled(Digit::D1);
        cell.clear().unwrap();
        assert_eq!(cell, CellState::Empty);

        let mut notes = DigitSet::new();
        notes.insert(Digit::D9);
        let mut cell = CellState::Notes(notes);
        cell.clear().unwrap();
        assert_eq!(cell, CellState::Empty);
    }

    #[test]
    fn test_cell_state_is_variant() {
        // derive_more::IsVariant generates these methods
        assert!(CellState::Given(Digit::D5).is_given());
        assert!(!CellState::Given(Digit::D5).is_filled());
        assert!(!CellState::Given(Digit::D5).is_empty());

        assert!(CellState::Filled(Digit::D5).is_filled());
        assert!(!CellState::Filled(Digit::D5).is_given());
        assert!(!CellState::Filled(Digit::D5).is_empty());

        assert!(CellState::Empty.is_empty());
        assert!(!CellState::Empty.is_given());
        assert!(!CellState::Empty.is_filled());
    }
}
