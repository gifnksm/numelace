//! Sudoku puzzle generator that creates puzzles with unique solutions.
//!
//! This crate provides functionality to generate valid Sudoku puzzles using the
//! removal method: it first generates a complete solution grid, then removes cells
//! one by one while ensuring the puzzle remains solvable using only logical deduction.
//!
//! # Algorithm
//!
//! The generator uses a two-step process:
//!
//! 1. **Generate a complete solution**: Creates a fully filled valid Sudoku grid
//!    using a hybrid approach of random placement and backtracking with solver assistance.
//!
//! 2. **Remove cells**: Attempts to remove as many cells as possible while maintaining
//!    unique solvability. The removal is verified using [`TechniqueSolver`] from the
//!    `sudoku-solver` crate, which ensures the puzzle can be solved using only human-like
//!    logical deduction techniques.
//!
//! # Key Property
//!
//! All generated puzzles are guaranteed to:
//! - Have exactly one solution
//! - Be solvable using only logical deduction (no guessing required)
//! - Be valid according to standard Sudoku rules
//!
//! # Examples
//!
//! ```
//! use sudoku_generator::PuzzleGenerator;
//! use sudoku_solver::TechniqueSolver;
//!
//! // Create a solver with all available techniques
//! let solver = TechniqueSolver::with_all_techniques();
//!
//! // Create a generator
//! let generator = PuzzleGenerator::new(&solver);
//!
//! // Generate a random puzzle
//! let puzzle = generator.generate();
//!
//! println!("Problem:\n{:#}", puzzle.problem);
//! println!("Solution:\n{:#}", puzzle.solution);
//! println!("Seed: {}", puzzle.seed);
//! ```
//!
//! # Reproducibility
//!
//! The generator supports reproducible generation via seeds:
//!
//! ```
//! use sudoku_generator::PuzzleGenerator;
//! use sudoku_solver::TechniqueSolver;
//!
//! let solver = TechniqueSolver::with_all_techniques();
//! let generator = PuzzleGenerator::new(&solver);
//!
//! // Generate a puzzle and save its seed
//! let puzzle1 = generator.generate();
//! let seed = puzzle1.seed;
//!
//! // Regenerate the exact same puzzle
//! let puzzle2 = generator.generate_with_seed(seed);
//!
//! assert_eq!(puzzle1.problem, puzzle2.problem);
//! assert_eq!(puzzle1.solution, puzzle2.solution);
//! ```

use std::fmt::Display;

use rand::{
    Rng, SeedableRng,
    distr::{Distribution, StandardUniform},
    seq::SliceRandom,
};
use rand_pcg::Pcg64;
use sudoku_core::{CandidateGrid, Digit, DigitGrid, DigitSet, Position};
use sudoku_solver::TechniqueSolver;

/// A Sudoku puzzle generator that creates puzzles with unique solutions.
///
/// The generator uses the removal method: it first generates a complete solution grid,
/// then removes cells one by one while verifying that the puzzle remains solvable
/// using only logical deduction techniques.
///
/// # Examples
///
/// ```
/// use sudoku_generator::PuzzleGenerator;
/// use sudoku_solver::TechniqueSolver;
///
/// let solver = TechniqueSolver::with_all_techniques();
/// let generator = PuzzleGenerator::new(&solver);
///
/// let puzzle = generator.generate();
/// ```
#[derive(Debug, Clone)]
pub struct PuzzleGenerator<'a> {
    solver: &'a TechniqueSolver,
}

impl<'a> PuzzleGenerator<'a> {
    /// Create a new generator with a solver
    #[must_use]
    pub fn new(solver: &'a TechniqueSolver) -> Self {
        Self { solver }
    }

    /// Generates a puzzle with a random seed.
    ///
    /// Each call produces a different puzzle. If you need reproducible generation,
    /// use [`generate_with_seed`](Self::generate_with_seed) instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_generator::PuzzleGenerator;
    /// use sudoku_solver::TechniqueSolver;
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let generator = PuzzleGenerator::new(&solver);
    ///
    /// let puzzle = generator.generate();
    /// ```
    #[must_use]
    pub fn generate(&self) -> GeneratedPuzzle {
        self.generate_with_seed(rand::random())
    }

    /// Generates a puzzle with a specific seed for reproducibility.
    ///
    /// The same seed will always produce the same puzzle, making it useful for:
    /// - Testing
    /// - Sharing specific puzzles
    /// - Debugging
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_generator::{PuzzleGenerator, PuzzleSeed};
    /// use sudoku_solver::TechniqueSolver;
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let generator = PuzzleGenerator::new(&solver);
    ///
    /// let puzzle1 = generator.generate();
    /// let seed = puzzle1.seed;
    ///
    /// // Generate the same puzzle again
    /// let puzzle2 = generator.generate_with_seed(seed);
    /// assert_eq!(puzzle1.problem, puzzle2.problem);
    /// ```
    #[must_use]
    pub fn generate_with_seed(&self, seed: PuzzleSeed) -> GeneratedPuzzle {
        let mut rng = Pcg64::from_seed(seed.0);
        let solution = self.generate_solution(&mut rng);
        let problem = self.remove_cells(&mut rng, &solution);
        GeneratedPuzzle {
            problem,
            solution,
            seed,
        }
    }

    /// Generates a complete, valid Sudoku solution grid.
    ///
    /// This method uses a hybrid approach:
    /// 1. Fill the first row with shuffled digits 1-9
    /// 2. Fill the remaining cells in the top-left box (avoiding the first row)
    /// 3. Use backtracking with solver assistance to fill the remaining cells
    ///
    /// The solver is used to eliminate obviously impossible candidates, making
    /// the backtracking more efficient.
    fn generate_solution<R>(&self, rng: &mut R) -> DigitGrid
    where
        R: Rng,
    {
        let mut grid = CandidateGrid::new();

        // Step 1: Fill the entire first row with shuffled digits 1-9
        let mut top_row = Digit::ALL;
        top_row.shuffle(rng);
        for (x, digit) in (0..9).zip(top_row) {
            let pos = Position::new(x, 0);
            grid.place(pos, digit);
        }

        // Step 2: Fill the remaining 6 cells in the top-left box (3x3)
        // top_row[0..3] are already used in the first row of the top-left box
        // top_row[3..9] contains the 6 digits unused in the top-left box
        let mut remaining: [Digit; 6] = top_row[3..9].try_into().unwrap();
        remaining.shuffle(rng);
        for (i, digit) in (3..9).zip(remaining) {
            let pos = Position::from_box(0, i); // Box 0, positions 3-8
            grid.place(pos, digit);
        }

        // Step 3: Fill the rest of the grid using backtracking with solver assistance
        let mut stack = vec![];
        let assumption = find_best_assumption(&grid);
        stack.push((grid, assumption));

        while let Some((mut grid, (pos, mut digits))) = stack.pop() {
            if digits.is_empty() {
                continue;
            }
            // Pick a random candidate digit and try it
            let digit = digits.pop_nth(rng.random_range(0..digits.len())).unwrap();
            stack.push((grid.clone(), (pos, digits)));
            grid.place(pos, digit);
            if grid.is_solved() {
                return grid.to_digit_grid();
            }
            // Use the solver to fill in cells that can be determined logically
            let Ok((solved, _)) = self.solver.solve(&mut grid) else {
                continue; // Contradiction found, backtrack
            };
            if solved {
                return grid.to_digit_grid();
            }
            // Pick the next cell to fill
            let assumption = find_best_assumption(&grid);
            stack.push((grid, assumption));
        }
        unreachable!("Failed to generate complete grid - this should never happen");
    }

    /// Removes cells from a complete solution to create a puzzle.
    ///
    /// This method attempts to remove as many cells as possible while ensuring
    /// the puzzle remains solvable using only logical deduction:
    ///
    /// 1. Shuffle all 81 cell positions
    /// 2. For each position, try removing the cell
    /// 3. Verify the puzzle is still solvable using `TechniqueSolver`
    /// 4. If solvable, keep the cell removed; otherwise, restore it
    ///
    /// The resulting puzzle has the maximum number of removed cells while
    /// maintaining a unique solution that can be found using human-like techniques.
    fn remove_cells<R>(&self, rng: &mut R, solution: &DigitGrid) -> DigitGrid
    where
        R: Rng,
    {
        let mut problem = solution.clone();
        let mut positions = Position::ALL;
        positions.shuffle(rng);
        for pos in positions {
            let mut removed = problem.clone();
            removed.set(pos, None);
            let mut test_grid = CandidateGrid::from_digit_grid(&removed);
            let result = self.solver.solve(&mut test_grid);
            if result.is_ok_and(|(solved, _)| solved) {
                problem = removed;
            }
        }
        problem
    }
}

/// Finds the cell with the minimum number of remaining candidates (MRV heuristic).
///
/// This heuristic is used during solution generation to choose which cell to fill next.
/// Choosing cells with fewer candidates reduces the branching factor in backtracking,
/// making the search more efficient.
///
/// # Returns
///
/// A tuple of `(Position, DigitSet)` where:
/// - `Position` is the cell with the fewest candidates
/// - `DigitSet` contains the valid candidate digits for that cell
///
/// # Panics
///
/// Panics if there are no undecided cells (should not happen during generation).
fn find_best_assumption(grid: &CandidateGrid) -> (Position, DigitSet) {
    // classify_cells::<10> groups cells by candidate count (0-9)
    // [0]: 0 candidates (contradiction), [1]: 1 candidate (decided), [2..]: 2-9 candidates
    let [empty, decided, cells @ ..] = grid.classify_cells::<10>();
    assert!(empty.is_empty() && decided.len() < 81);

    // Pick the first undecided cell with minimum candidates
    let pos = cells.iter().find_map(|cells| cells.first()).unwrap();
    (pos, grid.candidates_at(pos))
}

/// A 256-bit seed for reproducible puzzle generation.
///
/// The seed is used to initialize the random number generator, ensuring that
/// the same seed always produces the same puzzle.
///
/// # Display Format
///
/// The seed is displayed as a 64-character lowercase hexadecimal string:
///
/// ```
/// use sudoku_generator::PuzzleSeed;
///
/// let seed = PuzzleSeed::from([1u8; 32]);
/// let hex = format!("{}", seed);
/// assert_eq!(hex.len(), 64);
/// assert_eq!(hex, "0101010101010101010101010101010101010101010101010101010101010101");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct PuzzleSeed(pub [u8; 32]);

impl From<[u8; 32]> for PuzzleSeed {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl Display for PuzzleSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl Distribution<PuzzleSeed> for StandardUniform {
    fn sample<R>(&self, rng: &mut R) -> PuzzleSeed
    where
        R: Rng + ?Sized,
    {
        PuzzleSeed(rng.random())
    }
}

/// A generated Sudoku puzzle with its solution and seed.
///
/// This struct contains everything needed to work with a generated puzzle:
/// - The puzzle to solve (with some cells empty)
/// - The complete solution
/// - The seed used to generate it (for reproducibility)
#[derive(Debug, Clone)]
pub struct GeneratedPuzzle {
    /// The puzzle with some cells removed (the problem to solve).
    pub problem: DigitGrid,

    /// The complete solution grid.
    pub solution: DigitGrid,

    /// The seed used to generate this puzzle.
    ///
    /// Can be used with [`PuzzleGenerator::generate_with_seed`] to regenerate
    /// the exact same puzzle.
    pub seed: PuzzleSeed,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_generator() -> PuzzleGenerator<'static> {
        // Use a leaked solver to get a 'static reference
        // This is acceptable in tests
        let solver = Box::leak(Box::new(TechniqueSolver::with_all_techniques()));
        PuzzleGenerator::new(solver)
    }

    #[test]
    fn test_generated_solution_is_complete() {
        let generator = create_test_generator();
        let mut rng = Pcg64::from_seed([1u8; 32]);
        let solution = generator.generate_solution(&mut rng);

        // All cells should be filled
        for pos in Position::ALL {
            assert!(
                solution.get(pos).is_some(),
                "Cell at ({pos:?}) should be filled"
            );
        }
    }

    #[test]
    fn test_generated_solution_satisfies_sudoku_constraints() {
        let generator = create_test_generator();
        let mut rng = Pcg64::from_seed([2u8; 32]);
        let solution = generator.generate_solution(&mut rng);

        // Check all rows have digits 1-9
        for row in 0..9 {
            let mut digits = DigitSet::EMPTY;
            for pos in Position::ROWS[row] {
                if let Some(digit) = solution.get(pos) {
                    digits.insert(digit);
                }
            }
            assert_eq!(
                digits,
                DigitSet::FULL,
                "Row {row} should contain all digits 1-9"
            );
        }

        // Check all columns have digits 1-9
        for col in 0..9 {
            let mut digits = DigitSet::EMPTY;
            for pos in Position::COLUMNS[col] {
                if let Some(digit) = solution.get(pos) {
                    digits.insert(digit);
                }
            }
            assert_eq!(
                digits,
                DigitSet::FULL,
                "Column {col} should contain all digits 1-9"
            );
        }

        // Check all 3x3 boxes have digits 1-9
        for box_idx in 0..9 {
            let mut digits = DigitSet::EMPTY;
            for pos in Position::BOXES[box_idx] {
                if let Some(digit) = solution.get(pos) {
                    digits.insert(digit);
                }
            }
            assert_eq!(
                digits,
                DigitSet::FULL,
                "Box {box_idx} should contain all digits 1-9"
            );
        }
    }

    #[test]
    fn test_same_seed_produces_same_solution() {
        let generator = create_test_generator();
        let seed = [42u8; 32];

        let mut rng1 = Pcg64::from_seed(seed);
        let solution1 = generator.generate_solution(&mut rng1);

        let mut rng2 = Pcg64::from_seed(seed);
        let solution2 = generator.generate_solution(&mut rng2);

        // Same seed should produce identical solutions
        assert_eq!(solution1, solution2);
    }

    #[test]
    fn test_different_seeds_produce_different_solutions() {
        let generator = create_test_generator();

        let mut rng1 = Pcg64::from_seed([1u8; 32]);
        let solution1 = generator.generate_solution(&mut rng1);

        let mut rng2 = Pcg64::from_seed([2u8; 32]);
        let solution2 = generator.generate_solution(&mut rng2);

        // Different seeds should (almost certainly) produce different solutions
        assert_ne!(solution1, solution2);
    }

    #[test]
    fn test_generated_solution_can_be_verified_by_candidate_grid() {
        let generator = create_test_generator();
        let mut rng = Pcg64::from_seed([5u8; 32]);
        let solution = generator.generate_solution(&mut rng);

        // Verify the solution by placing it in a CandidateGrid
        let mut candidate_grid = CandidateGrid::new();
        for pos in Position::ALL {
            if let Some(digit) = solution.get(pos) {
                candidate_grid.place(pos, digit);
            }
        }

        // Should be completely solved without contradictions
        assert!(candidate_grid.is_solved());
    }

    #[test]
    fn test_removed_puzzle_solves_to_original_solution() {
        let generator = create_test_generator();
        let mut rng = Pcg64::from_seed([42u8; 32]);

        // Generate a complete solution
        let solution = generator.generate_solution(&mut rng);

        // Remove cells
        let problem = generator.remove_cells(&mut rng, &solution);

        // Solve the problem
        let mut test_grid = CandidateGrid::from_digit_grid(&problem);
        let result = generator.solver.solve(&mut test_grid);

        assert!(result.is_ok());
        assert!(result.unwrap().0); // solved

        // Verify the solution matches the original
        let solved_grid = test_grid.to_digit_grid();
        assert_eq!(
            solved_grid, solution,
            "Solved puzzle should match original solution"
        );
    }

    #[test]
    fn test_remove_cells_removes_at_least_some_cells() {
        let generator = create_test_generator();
        let mut rng = Pcg64::from_seed([100u8; 32]);

        let solution = generator.generate_solution(&mut rng);
        let problem = generator.remove_cells(&mut rng, &solution);

        // Count removed cells
        let removed_count = Position::ALL
            .iter()
            .filter(|&pos| solution.get(*pos).is_some() && problem.get(*pos).is_none())
            .count();

        assert!(removed_count > 0, "Should remove at least some cells");
        assert!(removed_count < 81, "Should not remove all cells");
    }

    #[test]
    fn test_remove_cells_problem_is_subset_of_solution() {
        let generator = create_test_generator();
        let mut rng = Pcg64::from_seed([7u8; 32]);

        let solution = generator.generate_solution(&mut rng);
        let problem = generator.remove_cells(&mut rng, &solution);

        // Every filled cell in problem should match the solution
        for pos in Position::ALL {
            if let Some(problem_digit) = problem.get(pos) {
                let solution_digit = solution.get(pos).unwrap();
                assert_eq!(
                    problem_digit, solution_digit,
                    "Problem cell at {pos:?} should match solution"
                );
            }
        }
    }

    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(10))]

            #[test]
            fn generated_puzzle_is_solvable(seed: [u8; 32]) {
                let generator = create_test_generator();
                let puzzle = generator.generate_with_seed(PuzzleSeed(seed));

                let mut test_grid = CandidateGrid::from_digit_grid(&puzzle.problem);
                let result = generator.solver.solve(&mut test_grid);

                prop_assert!(result.is_ok());
                prop_assert!(result.unwrap().0); // solved
            }

            #[test]
            fn same_seed_produces_same_puzzle(seed: [u8; 32]) {
                let generator = create_test_generator();
                let puzzle1 = generator.generate_with_seed(PuzzleSeed(seed));
                let puzzle2 = generator.generate_with_seed(PuzzleSeed(seed));

                prop_assert_eq!(puzzle1.problem, puzzle2.problem);
                prop_assert_eq!(puzzle1.solution, puzzle2.solution);
            }

            #[test]
            fn problem_is_subset_of_solution(seed: [u8; 32]) {
                let generator = create_test_generator();
                let puzzle = generator.generate_with_seed(PuzzleSeed(seed));

                for pos in Position::ALL {
                    if let Some(problem_digit) = puzzle.problem.get(pos) {
                        let solution_digit = puzzle.solution.get(pos).unwrap();
                        prop_assert_eq!(problem_digit, solution_digit);
                    }
                }
            }
        }
    }
}
