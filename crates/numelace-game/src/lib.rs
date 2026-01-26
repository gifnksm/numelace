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
    CandidateGrid, Digit, Position,
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
                CellState::Empty => {}
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
            cell @ CellState::Empty => *cell = CellState::Filled(digit),
        }
        Ok(())
    }

    /// Returns whether a digit can be placed at the given position.
    ///
    /// This returns `false` for given cells, which cannot be modified.
    #[must_use]
    pub fn can_set_digit(&self, pos: Position) -> bool {
        !self.cell(pos).is_given()
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
    /// game.remove_digit(empty_pos).unwrap();
    /// assert!(game.cell(empty_pos).is_empty());
    /// ```
    pub fn remove_digit(&mut self, pos: Position) -> Result<(), GameError> {
        match &mut self.grid[pos] {
            CellState::Given(_) => return Err(GameError::CannotModifyGivenCell),
            cell @ CellState::Filled(_) => *cell = CellState::Empty,
            CellState::Empty => {}
        }
        Ok(())
    }

    /// Returns whether a digit can be removed at the given position.
    ///
    /// This returns `false` for given cells, which cannot be modified.
    #[must_use]
    pub fn can_remove_digit(&self, pos: Position) -> bool {
        !self.cell(pos).is_given()
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
/// This enum distinguishes between three types of cells:
/// - [`Given`]: Initial puzzle cells (cannot be modified)
/// - [`Filled`]: Player-filled cells (can be modified or cleared)
/// - [`Empty`]: Cells that have not been filled yet
///
/// [`Given`]: CellState::Given
/// [`Filled`]: CellState::Filled
/// [`Empty`]: CellState::Empty
#[derive(Debug, Clone, PartialEq, Eq, derive_more::IsVariant)]
pub enum CellState {
    /// A cell from the initial puzzle (cannot be modified by the player).
    Given(Digit),
    /// A cell filled by the player (can be modified or cleared).
    Filled(Digit),
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
            CellState::Empty => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use numelace_core::{Digit, Position};
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
            game.remove_digit(given_pos),
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

        game.remove_digit(empty_pos).unwrap();
        assert!(game.cell(empty_pos).is_empty());

        // Clear empty cell is no-op
        assert!(game.remove_digit(empty_pos).is_ok());
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

        assert!(!game.can_set_digit(given_pos));
        assert!(game.can_set_digit(empty_pos));

        assert!(!game.can_remove_digit(given_pos));
        assert!(game.can_remove_digit(empty_pos));

        assert!(!game.has_removable_digit(empty_pos));
        game.set_digit(empty_pos, Digit::D5).unwrap();
        assert!(game.has_removable_digit(empty_pos));
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

        // as_digit (unified access)
        assert_eq!(CellState::Given(Digit::D5).as_digit(), Some(Digit::D5));
        assert_eq!(CellState::Filled(Digit::D7).as_digit(), Some(Digit::D7));
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
