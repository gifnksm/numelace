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
//! use numelace_game::{Game, CellState};
//! use numelace_generator::PuzzleGenerator;
//! use numelace_solver::TechniqueSolver;
//! use numelace_core::{Digit, Position};
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
//! let empty_pos = *Position::ALL.iter()
//!     .find(|&&pos| game.cell(pos).is_empty())
//!     .expect("puzzle has empty cells");
//!
//! // Fill it with a digit
//! game.set_digit(empty_pos, Digit::D5).unwrap();
//!
//! // Check if solved
//! if game.is_solved() {
//!     println!("Puzzle completed!");
//! }
//! ```
//!

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
#[derive(Debug, Clone)]
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
                this.set_digit(pos, digit)?;
            }
        }

        for (y, row) in (0..9).zip(notes) {
            for (x, bits) in (0..9).zip(row) {
                let pos = Position::new(x, y);
                let digits =
                    DigitSet::try_from_bits(*bits).ok_or(GameError::InvalidNotes(*bits))?;
                for d in digits {
                    this.toggle_note(pos, d)?;
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
    /// use numelace_game::{Game, CellState};
    /// use numelace_generator::PuzzleGenerator;
    /// use numelace_solver::TechniqueSolver;
    /// use numelace_core::Position;
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
    /// use numelace_game::Game;
    /// use numelace_generator::PuzzleGenerator;
    /// use numelace_solver::TechniqueSolver;
    /// use numelace_core::{Digit, Position};
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
    ///         game.set_digit(pos, digit).unwrap();
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

    /// Places a digit at the given position.
    ///
    /// If the cell is empty, it becomes filled. If the cell is already filled,
    /// the digit is replaced.
    ///
    /// # Errors
    ///
    /// Returns [`GameError::CannotModifyGivenCell`] if the position contains a given cell.
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_game::Game;
    /// use numelace_generator::PuzzleGenerator;
    /// use numelace_solver::TechniqueSolver;
    /// use numelace_core::{Digit, Position};
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let generator = PuzzleGenerator::new(&solver);
    /// let puzzle = generator.generate();
    /// let mut game = Game::new(puzzle);
    ///
    /// // Find an empty cell
    /// let empty_pos = *Position::ALL.iter()
    ///     .find(|&&pos| game.cell(pos).is_empty())
    ///     .expect("puzzle has empty cells");
    ///
    /// // Fill it
    /// game.set_digit(empty_pos, Digit::D5).unwrap();
    /// assert_eq!(game.cell(empty_pos).as_digit(), Some(Digit::D5));
    /// ```
    pub fn set_digit(&mut self, pos: Position, digit: Digit) -> Result<(), GameError> {
        match &mut self.grid[pos] {
            CellState::Given(_) => return Err(GameError::CannotModifyGivenCell),
            CellState::Filled(d) => *d = digit,
            cell @ (CellState::Notes(_) | CellState::Empty) => *cell = CellState::Filled(digit),
        }
        Ok(())
    }

    /// Toggles a digit at the given position.
    ///
    /// If the cell is empty, it becomes filled with the digit. If the cell is
    /// already filled with the same digit, it becomes empty. If the cell is
    /// filled with a different digit, the digit is replaced.
    ///
    /// # Errors
    ///
    /// Returns [`GameError::CannotModifyGivenCell`] if the position contains a given cell.
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_game::Game;
    /// use numelace_generator::PuzzleGenerator;
    /// use numelace_solver::TechniqueSolver;
    /// use numelace_core::{Digit, Position};
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let generator = PuzzleGenerator::new(&solver);
    /// let puzzle = generator.generate();
    /// let mut game = Game::new(puzzle);
    ///
    /// // Find an empty cell
    /// let empty_pos = *Position::ALL.iter()
    ///     .find(|&&pos| game.cell(pos).is_empty())
    ///     .expect("puzzle has empty cells");
    ///
    /// // Toggle on
    /// game.toggle_digit(empty_pos, Digit::D5).unwrap();
    /// assert_eq!(game.cell(empty_pos).as_digit(), Some(Digit::D5));
    ///
    /// // Toggle off
    /// game.toggle_digit(empty_pos, Digit::D5).unwrap();
    /// assert!(game.cell(empty_pos).is_empty());
    /// ```
    pub fn toggle_digit(&mut self, pos: Position, digit: Digit) -> Result<(), GameError> {
        if self.grid[pos].as_digit() == Some(digit) {
            self.clear_cell(pos)
        } else {
            self.set_digit(pos, digit)
        }
    }

    /// Returns whether a digit can be placed at the given position.
    ///
    /// This returns `false` for given cells, which cannot be modified.
    #[must_use]
    pub fn can_toggle_digit(&self, pos: Position) -> bool {
        !self.cell(pos).is_given()
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
    pub fn toggle_note(&mut self, pos: Position, digit: Digit) -> Result<(), GameError> {
        match &mut self.grid[pos] {
            CellState::Given(_) => return Err(GameError::CannotModifyGivenCell),
            CellState::Filled(_) => return Err(GameError::CannotAddNoteToFilledCell),
            CellState::Notes(digits) => {
                digits.toggle(digit);
                if digits.is_empty() {
                    self.grid[pos] = CellState::Empty;
                }
            }
            cell @ CellState::Empty => {
                let mut digits = DigitSet::new();
                digits.insert(digit);
                *cell = CellState::Notes(digits);
            }
        }
        Ok(())
    }

    /// Returns whether a note can be toggled at the given position.
    ///
    /// This returns `false` for given and filled cells.
    #[must_use]
    pub fn can_toggle_note(&self, pos: Position) -> bool {
        match self.cell(pos) {
            CellState::Given(_) | CellState::Filled(_) => false,
            CellState::Notes(_) | CellState::Empty => true,
        }
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
    /// use numelace_game::Game;
    /// use numelace_generator::PuzzleGenerator;
    /// use numelace_solver::TechniqueSolver;
    /// use numelace_core::{Digit, Position};
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let generator = PuzzleGenerator::new(&solver);
    /// let puzzle = generator.generate();
    /// let mut game = Game::new(puzzle);
    ///
    /// // Find an empty cell and fill it
    /// let empty_pos = *Position::ALL.iter()
    ///     .find(|&&pos| game.cell(pos).is_empty())
    ///     .expect("puzzle has empty cells");
    /// game.set_digit(empty_pos, Digit::D5).unwrap();
    ///
    /// // Clear it
    /// game.clear_cell(empty_pos).unwrap();
    /// assert!(game.cell(empty_pos).is_empty());
    /// ```
    pub fn clear_cell(&mut self, pos: Position) -> Result<(), GameError> {
        match &mut self.grid[pos] {
            CellState::Given(_) => return Err(GameError::CannotModifyGivenCell),
            cell @ (CellState::Filled(_) | CellState::Notes(_)) => *cell = CellState::Empty,
            CellState::Empty => {}
        }
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
    /// Returns the digit if this is a given cell, otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_game::CellState;
    /// use numelace_core::Digit;
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
    /// use numelace_game::CellState;
    /// use numelace_core::Digit;
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
    /// use numelace_game::CellState;
    /// use numelace_core::Digit;
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
    use super::*;
    use numelace_core::{Digit, DigitGrid, DigitSet, Position};
    use numelace_generator::PuzzleGenerator;

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
        assert!(game.set_digit(empty_pos, Digit::D5).is_ok());
        assert_eq!(game.cell(empty_pos), &CellState::Filled(Digit::D5));

        // Can replace filled cell
        assert!(game.set_digit(empty_pos, Digit::D7).is_ok());
        assert_eq!(game.cell(empty_pos), &CellState::Filled(Digit::D7));
    }

    #[test]
    fn test_toggle_digit_basic_operations() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        let empty_pos = *Position::ALL
            .iter()
            .find(|&&pos| game.cell(pos).is_empty())
            .expect("puzzle has empty cells");

        // Toggle on
        assert!(game.toggle_digit(empty_pos, Digit::D5).is_ok());
        assert_eq!(game.cell(empty_pos), &CellState::Filled(Digit::D5));

        // Toggle off
        assert!(game.toggle_digit(empty_pos, Digit::D5).is_ok());
        assert!(game.cell(empty_pos).is_empty());

        // Replace with a different digit
        assert!(game.toggle_digit(empty_pos, Digit::D3).is_ok());
        assert!(game.toggle_digit(empty_pos, Digit::D7).is_ok());
        assert_eq!(game.cell(empty_pos), &CellState::Filled(Digit::D7));
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

        assert!(game.toggle_note(empty_pos, Digit::D4).is_ok());
        assert!(matches!(
            game.cell(empty_pos),
            CellState::Notes(digits) if digits.contains(Digit::D4)
        ));

        assert!(game.toggle_note(empty_pos, Digit::D4).is_ok());
        assert!(game.cell(empty_pos).is_empty());

        game.set_digit(empty_pos, Digit::D7).unwrap();
        assert!(matches!(
            game.toggle_note(empty_pos, Digit::D3),
            Err(GameError::CannotAddNoteToFilledCell)
        ));

        assert!(matches!(
            game.toggle_note(given_pos, Digit::D3),
            Err(GameError::CannotModifyGivenCell)
        ));
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
            game.set_digit(given_pos, Digit::D5),
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
        game.set_digit(empty_pos, Digit::D5).unwrap();
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

        assert!(!game.can_toggle_digit(given_pos));
        assert!(game.can_toggle_digit(empty_pos));
        assert!(!game.can_toggle_note(given_pos));
        assert!(game.can_toggle_note(empty_pos));

        assert!(!game.has_removable_digit(empty_pos));
        game.set_digit(empty_pos, Digit::D5).unwrap();
        assert!(game.has_removable_digit(empty_pos));
        assert!(!game.can_toggle_note(empty_pos));
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
        game.set_digit(first, Digit::D5).unwrap();
        game.set_digit(second, Digit::D5).unwrap();

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
                game.set_digit(pos, digit).unwrap();
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
                let _ = game.set_digit(pos, Digit::D1);
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
