use numelace_core::{
    CandidateGrid, Digit, DigitGrid, DigitSet, Position,
    containers::{Array9, Array81},
    index::{DigitSemantics, PositionSemantics},
};
use numelace_generator::GeneratedPuzzle;

use crate::{
    CellState, GameError, InputBlockReason, InputDigitOptions, InputOperation, RuleCheckPolicy,
};

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
    ///         let digit = puzzle.solution[pos].expect("solution is complete");
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
    ) -> Result<InputOperation, GameError> {
        let operation = self.cell(pos).set_digit_capability(digit)?;

        match operation {
            InputOperation::NoOp => return Ok(InputOperation::NoOp),
            InputOperation::Removed => {
                unreachable!("set_digit should not yield Removed");
            }
            InputOperation::Set => {}
        }

        if options.rule_check_policy.is_strict() && self.is_conflicting(pos, digit) {
            return Err(GameError::ConflictingDigit);
        }

        self.grid[pos].set_filled(digit)?;

        if options.note_cleanup_policy.is_remove_peers() {
            for peer_pos in pos.house_peers() {
                self.grid[peer_pos].drop_note_digit(digit);
            }
        }

        Ok(InputOperation::Set)
    }

    /// Returns the capability for placing a digit at the given position.
    ///
    /// The returned result indicates the cell-local operation or why it is blocked,
    /// taking the provided policy into account.
    ///
    /// # Errors
    ///
    /// Returns [`InputBlockReason::GivenCell`] if the cell is a given cell.
    /// Returns [`InputBlockReason::Conflict`] if strict rule checks are enabled and
    /// the digit conflicts with existing digits.
    pub fn set_digit_capability(
        &self,
        pos: Position,
        digit: Digit,
        policy: RuleCheckPolicy,
    ) -> Result<InputOperation, InputBlockReason> {
        let operation = self.cell(pos).set_digit_capability(digit)?;

        if matches!(operation, InputOperation::Set)
            && policy.is_strict()
            && self.is_conflicting(pos, digit)
        {
            return Err(InputBlockReason::Conflict);
        }

        Ok(operation)
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
    ) -> Result<InputOperation, GameError> {
        let operation = self.cell(pos).toggle_note_capability(digit)?;

        match operation {
            InputOperation::NoOp => return Ok(InputOperation::NoOp),
            InputOperation::Removed => {
                self.grid[pos].drop_note_digit(digit);
                return Ok(InputOperation::Removed);
            }
            InputOperation::Set => {}
        }

        if policy.is_strict() && self.is_conflicting(pos, digit) {
            return Err(GameError::ConflictingDigit);
        }

        self.grid[pos].add_note_digit(digit);
        Ok(InputOperation::Set)
    }

    /// Returns the toggle capability for notes at the given position.
    ///
    /// The returned result indicates the cell-local operation or why it is blocked,
    /// taking the provided policy into account.
    /// Note removal returns `Ok(InputOperation::Removed)` even under strict checks.
    ///
    /// # Errors
    ///
    /// Returns [`InputBlockReason::GivenCell`] if the cell is a given cell.
    /// Returns [`InputBlockReason::FilledCell`] if the cell is filled.
    /// Returns [`InputBlockReason::Conflict`] if strict rule checks are enabled and
    /// the digit conflicts with existing digits when adding a note.
    pub fn toggle_note_capability(
        &self,
        pos: Position,
        digit: Digit,
        policy: RuleCheckPolicy,
    ) -> Result<InputOperation, InputBlockReason> {
        let operation = self.cell(pos).toggle_note_capability(digit)?;

        if matches!(operation, InputOperation::Set)
            && policy.is_strict()
            && self.is_conflicting(pos, digit)
        {
            return Err(InputBlockReason::Conflict);
        }

        Ok(operation)
    }
    /// Returns the note auto-fill capability for a single cell.
    ///
    /// This computes candidate notes by excluding digits already present in peers,
    /// then reports whether applying those notes would be a no-op or a set.
    ///
    /// # Errors
    ///
    /// Returns [`InputBlockReason::GivenCell`] if the cell is a given cell.
    /// Returns [`InputBlockReason::FilledCell`] if the cell is filled.
    pub fn auto_fill_cell_notes_capability(
        &self,
        pos: Position,
    ) -> Result<InputOperation, InputBlockReason> {
        self.cell(pos).can_set_notes()?;
        let mut notes = DigitSet::FULL;
        for peer_pos in pos.house_peers() {
            if let Some(digit) = self.grid[peer_pos].as_digit() {
                notes.remove(digit);
            }
        }
        self.cell(pos).set_notes_capability(notes)
    }

    /// Auto-fills notes for a single cell by replacing its notes with computed candidates.
    ///
    /// Candidates are derived by excluding digits already present in peers. Empty candidates
    /// clear notes for the cell.
    ///
    /// # Errors
    ///
    /// Returns [`InputBlockReason::GivenCell`] if the cell is a given cell.
    /// Returns [`InputBlockReason::FilledCell`] if the cell is filled.
    pub fn auto_fill_cell_notes(
        &mut self,
        pos: Position,
    ) -> Result<InputOperation, InputBlockReason> {
        self.cell(pos).can_set_notes()?;
        let mut notes = DigitSet::FULL;
        for peer_pos in pos.house_peers() {
            if let Some(digit) = self.grid[peer_pos].as_digit() {
                notes.remove(digit);
            }
        }
        let operation = self.cell(pos).set_notes_capability(notes)?;
        match operation {
            InputOperation::NoOp => {}
            InputOperation::Set => {
                self.grid[pos].set_notes(notes);
            }
            InputOperation::Removed => unreachable!(""),
        }
        Ok(operation)
    }

    /// Auto-fills notes for all cells that can accept notes.
    ///
    /// Cells that cannot accept notes (given/filled) are skipped.
    pub fn auto_fill_notes_all_cells(&mut self) {
        for pos in Position::ALL {
            if self.cell(pos).can_set_notes().is_err() {
                continue;
            }
            #[expect(clippy::missing_panics_doc)]
            self.auto_fill_cell_notes(pos).unwrap();
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

    /// Returns whether the cell currently contains removable player input.
    ///
    /// This is `true` for filled (player-entered) digits or notes.
    #[must_use]
    pub fn has_removable_input(&self, pos: Position) -> bool {
        self.cell(pos).has_removable_input()
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

#[cfg(test)]
mod tests {
    use numelace_core::{Digit, DigitGrid, DigitSet, Position};
    use numelace_generator::PuzzleGenerator;

    use crate::NoteCleanupPolicy;

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

        game.set_digit(
            empty_pos,
            Digit::D5,
            &InputDigitOptions::default().note_cleanup_policy(NoteCleanupPolicy::None),
        )
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

        let empty_pos = *Position::ALL
            .iter()
            .find(|&&pos| game.cell(pos).is_empty())
            .expect("puzzle has empty cells");

        // Add note to empty cell
        game.toggle_note(empty_pos, Digit::D3, RuleCheckPolicy::Permissive)
            .unwrap();
        assert!(matches!(
            game.cell(empty_pos),
            CellState::Notes(notes) if notes.contains(Digit::D3)
        ));

        // Remove note
        game.toggle_note(empty_pos, Digit::D3, RuleCheckPolicy::Permissive)
            .unwrap();
        assert_eq!(game.cell(empty_pos), &CellState::Empty);
    }

    #[test]
    fn test_auto_fill_cell_notes_sets_candidates() {
        let problem: DigitGrid = "\
.12......\
3........\
.4.......\
.........\
.........\
.........\
.........\
.........\
.........\
"
        .parse()
        .expect("valid problem grid");
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
        .expect("valid filled grid");

        let mut game = Game::from_problem_filled_notes(&problem, &filled, &[[0; 9]; 9]).unwrap();
        let pos = Position::new(0, 0);

        let result = game.auto_fill_cell_notes(pos).unwrap();
        assert_eq!(result, InputOperation::Set);

        let mut expected = DigitSet::new();
        for digit in [Digit::D5, Digit::D6, Digit::D7, Digit::D8, Digit::D9] {
            expected.insert(digit);
        }

        assert!(matches!(
            game.cell(pos),
            CellState::Notes(notes) if *notes == expected
        ));
    }

    #[test]
    fn test_auto_fill_cell_notes_clears_when_no_candidates() {
        let problem: DigitGrid = "\
.12345678\
9........\
.........\
.........\
.........\
.........\
.........\
.........\
.........\
"
        .parse()
        .expect("valid problem grid");
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
        .expect("valid filled grid");

        let mut game = Game::from_problem_filled_notes(&problem, &filled, &[[0; 9]; 9]).unwrap();
        let pos = Position::new(0, 0);

        game.toggle_note(pos, Digit::D1, RuleCheckPolicy::Permissive)
            .unwrap();
        let result = game.auto_fill_cell_notes(pos).unwrap();
        assert_eq!(result, InputOperation::Set);
        assert_eq!(game.cell(pos), &CellState::Empty);
    }

    #[test]
    fn test_strict_conflict_rejects_inputs() {
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

        game.set_digit(first, Digit::D5, &InputDigitOptions::default())
            .unwrap();

        // Strict conflict rejects same digit in peer cell
        let result = game.set_digit(
            second,
            Digit::D5,
            &InputDigitOptions::default().rule_check_policy(RuleCheckPolicy::Strict),
        );
        assert!(matches!(result, Err(GameError::ConflictingDigit)));

        // Notes also rejected under strict conflict when adding
        let result = game.toggle_note(second, Digit::D5, RuleCheckPolicy::Strict);
        assert!(matches!(result, Err(GameError::ConflictingDigit)));

        // Removing note is always allowed
        game.toggle_note(second, Digit::D4, RuleCheckPolicy::Permissive)
            .unwrap();
        let result = game.toggle_note(second, Digit::D4, RuleCheckPolicy::Strict);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cannot_modify_given_cells() {
        use numelace_solver::TechniqueSolver;
        let solver = TechniqueSolver::with_all_techniques();
        let generator = PuzzleGenerator::new(&solver);
        let puzzle = generator.generate();
        let mut game = Game::new(puzzle);

        let given_pos = Position::ALL
            .into_iter()
            .find(|&pos| game.cell(pos).is_given())
            .expect("puzzle has given cells");

        assert!(matches!(
            game.set_digit(given_pos, Digit::D1, &InputDigitOptions::default()),
            Err(GameError::CannotModifyGivenCell)
        ));
        assert!(matches!(
            game.toggle_note(given_pos, Digit::D1, RuleCheckPolicy::Permissive),
            Err(GameError::CannotModifyGivenCell)
        ));
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
            Err(InputBlockReason::GivenCell)
        );
        assert_eq!(
            game.toggle_note_capability(given_pos, Digit::D1, RuleCheckPolicy::Permissive),
            Err(InputBlockReason::GivenCell)
        );
        assert_eq!(
            game.set_digit_capability(empty_pos, Digit::D1, RuleCheckPolicy::Permissive),
            Ok(InputOperation::Set)
        );
        assert_eq!(
            game.toggle_note_capability(empty_pos, Digit::D1, RuleCheckPolicy::Permissive),
            Ok(InputOperation::Set)
        );

        game.set_digit(empty_pos, Digit::D5, &InputDigitOptions::default())
            .unwrap();
        assert_eq!(
            game.toggle_note_capability(empty_pos, Digit::D1, RuleCheckPolicy::Permissive),
            Err(InputBlockReason::FilledCell)
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
}
