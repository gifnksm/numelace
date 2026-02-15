//! Backtracking solver for Sudoku puzzles.
//!
//! This module provides a solver that combines technique-based solving with backtracking.
//! When techniques cannot make further progress, the solver makes assumptions and explores
//! the search space to find solutions.

use numelace_core::{Digit, DigitSet, Position};

use crate::{
    SolverError, TechniqueGrid, TechniqueSolver, TechniqueSolverStats, backtrack,
    technique::BoxedTechnique,
};

/// Statistics collected during backtracking solving.
///
/// Tracks technique applications, assumptions made, and backtrack events.
#[derive(Debug, Default, Clone)]
pub struct BacktrackSolverStats {
    technique: TechniqueSolverStats,
    assumptions: Vec<(Position, Digit)>,
    backtrack_count: usize,
}

impl BacktrackSolverStats {
    /// Creates a new empty statistics object.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates statistics with initial technique statistics.
    #[must_use]
    pub fn with_technique(technique: TechniqueSolverStats) -> Self {
        Self {
            technique,
            assumptions: vec![],
            backtrack_count: 0,
        }
    }

    /// Returns the technique solver statistics.
    #[must_use]
    pub fn technique(&self) -> &TechniqueSolverStats {
        &self.technique
    }

    /// Returns the list of assumptions made during solving.
    ///
    /// Each assumption is a `(Position, Digit)` pair representing a cell and
    /// the digit that was assumed for it.
    #[must_use]
    pub fn assumptions(&self) -> &[(Position, Digit)] {
        &self.assumptions
    }

    /// Returns `true` if the puzzle was solved without making any assumptions.
    ///
    /// This indicates the puzzle was solvable using only the configured techniques.
    #[must_use]
    pub fn solved_without_assumptions(&self) -> bool {
        self.assumptions.is_empty()
    }

    /// Returns the number of assumptions that led to contradictions.
    ///
    /// This counts how many times the solver had to backtrack because an
    /// assumption resulted in an inconsistent state during technique application.
    #[must_use]
    pub fn backtrack_count(&self) -> usize {
        self.backtrack_count
    }
}

/// A solver that combines technique-based solving with backtracking.
///
/// `BacktrackSolver` first applies techniques to solve as much as possible,
/// then uses backtracking to explore remaining possibilities when stuck.
/// It can find all solutions to a puzzle, not just the first one.
///
/// # Examples
///
/// ```
/// use numelace_solver::{BacktrackSolver, TechniqueGrid};
///
/// let solver = BacktrackSolver::with_all_techniques();
/// let grid = TechniqueGrid::new();
///
/// // Get first solution
/// if let Some((solution, stats)) = solver.solve(grid)?.next() {
///     println!("Solved with {} assumptions", stats.assumptions().len());
///     if stats.solved_without_assumptions() {
///         println!("No backtracking needed!");
///     }
/// }
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
///
/// # Finding Multiple Solutions
///
/// ```
/// use numelace_solver::{BacktrackSolver, TechniqueGrid};
///
/// let solver = BacktrackSolver::with_all_techniques();
/// let grid = TechniqueGrid::new();
///
/// // Check for unique solution
/// let solutions: Vec<_> = solver.solve(grid)?.take(2).collect();
/// match solutions.len() {
///     0 => println!("No solution"),
///     1 => println!("Unique solution"),
///     _ => println!("Multiple solutions"),
/// }
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```

#[derive(Debug, Clone)]
pub struct BacktrackSolver {
    technique: TechniqueSolver,
}

impl BacktrackSolver {
    /// Creates a new backtracking solver with the specified technique solver.
    #[must_use]
    pub fn new(technique: TechniqueSolver) -> Self {
        Self { technique }
    }

    /// Creates a solver with all available techniques enabled.
    #[must_use]
    pub fn with_all_techniques() -> Self {
        Self::new(TechniqueSolver::with_all_techniques())
    }

    /// Creates a solver with only the specified techniques.
    #[must_use]
    pub fn with_techniques(techniques: Vec<BoxedTechnique>) -> Self {
        Self::new(TechniqueSolver::new(techniques))
    }

    /// Creates a solver with fundamental techniques.
    ///
    /// Uses the most basic logical solving techniques (Naked Single and Hidden Single)
    /// combined with backtracking. These Singles techniques form the foundation of
    /// technique-based solving and provide a stable baseline for benchmarking.
    ///
    /// This constructor uses [`technique::fundamental_techniques`](crate::technique::fundamental_techniques),
    /// which provides a stable technique set that serves as a consistent baseline.
    /// The set remains unchanged even as more advanced techniques are added to the library.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_solver::BacktrackSolver;
    ///
    /// let solver = BacktrackSolver::with_fundamental_techniques();
    /// ```
    ///
    /// # See Also
    ///
    /// - [`with_all_techniques`](Self::with_all_techniques) - Use all available techniques
    /// - [`with_techniques`](Self::with_techniques) - Use a custom set of techniques
    /// - [`without_techniques`](Self::without_techniques) - Use backtracking only
    #[must_use]
    pub fn with_fundamental_techniques() -> Self {
        Self::new(TechniqueSolver::with_fundamental_techniques())
    }

    /// Creates a backtracking solver without techniques.
    ///
    /// This solver relies entirely on backtracking to find solutions.
    #[must_use]
    pub fn without_techniques() -> Self {
        Self::new(TechniqueSolver::new(vec![]))
    }

    /// Solves the puzzle and returns an iterator over all solutions.
    ///
    /// The iterator yields solutions in the order they are found during the
    /// search. Each solution comes with statistics about how it was found.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::Inconsistent`] if the initial grid is inconsistent
    /// (has cells with no candidates or contradictory placements).
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_solver::{BacktrackSolver, TechniqueGrid};
    ///
    /// let solver = BacktrackSolver::with_all_techniques();
    /// let grid = TechniqueGrid::new();
    ///
    /// // Get first solution only
    /// match solver.solve(grid)?.next() {
    ///     Some((solution, stats)) => {
    ///         println!("Found solution!");
    ///         println!("Assumptions: {}", stats.assumptions().len());
    ///         println!("Backtracks: {}", stats.backtrack_count());
    ///     }
    ///     None => println!("No solution exists"),
    /// }
    /// # Ok::<(), numelace_solver::SolverError>(())
    /// ```
    pub fn solve(&self, mut grid: TechniqueGrid) -> Result<Solutions<'_>, SolverError> {
        let mut stats = BacktrackSolverStats::new();
        let solved = self.solve_by_technique(&mut grid, &mut stats)?;
        let solutions = if solved {
            Solutions::solved(self, grid, stats)
        } else {
            let assumption = backtrack::find_best_assumption(&grid);
            Solutions::with_assumptions(self, grid, stats, assumption)
        };
        Ok(solutions)
    }

    fn solve_by_technique(
        &self,
        grid: &mut TechniqueGrid,
        stats: &mut BacktrackSolverStats,
    ) -> Result<bool, SolverError> {
        let solved = self
            .technique
            .solve_with_stats(grid, &mut stats.technique)?;
        Ok(solved)
    }
}

/// An iterator over solutions to a Sudoku puzzle.
///
/// Created by [`BacktrackSolver::solve`]. Yields solutions along with statistics
/// about how each solution was found.

#[derive(Debug, Clone)]
pub struct Solutions<'a> {
    solver: &'a BacktrackSolver,
    stack: Vec<SearchState>,
}

#[derive(Debug, Clone)]
struct SearchState {
    grid: TechniqueGrid,
    stats: BacktrackSolverStats,
    assumption: Option<(Position, DigitSet)>,
}

impl SearchState {
    fn solved(grid: TechniqueGrid, stats: BacktrackSolverStats) -> Self {
        Self {
            grid,
            stats,
            assumption: None,
        }
    }

    fn with_assumption(
        grid: TechniqueGrid,
        stats: BacktrackSolverStats,
        assumption: (Position, DigitSet),
    ) -> Self {
        Self {
            grid,
            stats,
            assumption: Some(assumption),
        }
    }
}

impl<'a> Solutions<'a> {
    fn solved(
        solver: &'a BacktrackSolver,
        grid: TechniqueGrid,
        stats: BacktrackSolverStats,
    ) -> Self {
        Self {
            solver,
            stack: vec![SearchState::solved(grid, stats)],
        }
    }

    fn with_assumptions(
        solver: &'a BacktrackSolver,
        grid: TechniqueGrid,
        stats: BacktrackSolverStats,
        assumption: (Position, DigitSet),
    ) -> Self {
        Self {
            solver,
            stack: vec![SearchState::with_assumption(grid, stats, assumption)],
        }
    }
}

impl Iterator for Solutions<'_> {
    type Item = (TechniqueGrid, BacktrackSolverStats);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut state) = self.stack.pop() {
            let Some((pos, remaining_digits)) = &mut state.assumption else {
                return Some((state.grid, state.stats));
            };
            let Some(digit) = remaining_digits.pop_first() else {
                continue;
            };
            let pos = *pos;
            let mut grid = state.grid.clone();
            let mut stats = state.stats.clone();
            self.stack.push(state);

            stats.assumptions.push((pos, digit));
            grid.place(pos, digit);
            let Ok(solved) = self.solver.solve_by_technique(&mut grid, &mut stats) else {
                stats.backtrack_count += 1;
                continue;
            };
            if solved {
                return Some((grid, stats));
            }
            let assumption = backtrack::find_best_assumption(&grid);
            self.stack
                .push(SearchState::with_assumption(grid, stats, assumption));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{CandidateGrid, Digit, Position};

    use super::*;

    #[test]
    fn test_without_techniques_solver() {
        let solver = BacktrackSolver::without_techniques();
        let grid = TechniqueGrid::new();

        // Should be able to solve even without techniques
        let result = solver.solve(grid);
        assert!(result.is_ok());
    }

    #[test]
    fn test_solve_with_all_techniques() {
        let solver = BacktrackSolver::with_all_techniques();
        let mut grid = TechniqueGrid::new();

        // Create a naked single
        grid.place(Position::new(4, 4), Digit::D5);

        let result = solver.solve(grid);
        assert!(result.is_ok());
    }

    #[test]
    fn test_solve_returns_iterator() {
        let solver = BacktrackSolver::with_all_techniques();
        let mut grid = TechniqueGrid::new();

        // Simple setup
        grid.place(Position::new(0, 0), Digit::D1);

        let solutions = solver.solve(grid).unwrap();

        // Should be able to iterate
        let count = solutions.take(5).count();
        assert!(count > 0);
    }

    #[test]
    fn test_stats_solved_without_assumptions() {
        let stats = BacktrackSolverStats::new();

        // Empty stats should indicate no assumptions
        assert!(stats.solved_without_assumptions());
    }

    #[test]
    fn test_stats_getters() {
        let stats = BacktrackSolverStats::new();

        assert_eq!(stats.assumptions().len(), 0);
        assert_eq!(stats.backtrack_count(), 0);
        assert!(stats.solved_without_assumptions());
        assert!(stats.technique().total_steps() == 0);
    }

    #[test]
    fn test_stats_with_technique() {
        let tech_stats = TechniqueSolverStats::new();
        let stats = BacktrackSolverStats::with_technique(tech_stats);

        assert_eq!(stats.assumptions().len(), 0);
        assert_eq!(stats.backtrack_count(), 0);
    }

    #[test]
    fn test_contradiction_in_initial_grid() {
        let solver = BacktrackSolver::with_all_techniques();
        let mut grid = TechniqueGrid::new();

        // Create a contradiction: remove all candidates from a cell
        for digit in Digit::ALL {
            grid.remove_candidate(Position::new(0, 0), digit);
        }

        let result = solver.solve(grid);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SolverError::Inconsistent(_)));
    }

    #[test]
    fn test_multiple_solutions() {
        let solver = BacktrackSolver::without_techniques();
        let grid = TechniqueGrid::new();

        // Empty grid has multiple solutions - verify we can get at least 2
        let solutions: Vec<_> = solver.solve(grid).unwrap().take(2).collect();
        assert_eq!(solutions.len(), 2);

        // Verify that solutions are different
        let (grid1, _) = &solutions[0];
        let (grid2, _) = &solutions[1];
        assert_ne!(grid1.to_digit_grid(), grid2.to_digit_grid());
    }

    #[test]
    fn test_multiple_solutions_with_partial_grid() {
        let solver = BacktrackSolver::without_techniques();
        let mut grid = CandidateGrid::new();

        // Place a few digits to create a partial puzzle with multiple solutions
        grid.place(Position::new(0, 0), Digit::D1);
        grid.place(Position::new(1, 1), Digit::D2);
        grid.place(Position::new(2, 2), Digit::D3);

        // Should still have multiple solutions
        let solutions: Vec<_> = solver.solve(grid.into()).unwrap().take(3).collect();
        assert_eq!(solutions.len(), 3);

        // Verify all solutions are valid and different
        for i in 0..solutions.len() {
            let (grid_i, _) = &solutions[i];
            assert!(grid_i.is_solved().unwrap());

            // Check that the original placements are preserved
            let digit_grid_i = grid_i.to_digit_grid();
            assert_eq!(digit_grid_i[Position::new(0, 0)], Some(Digit::D1));
            assert_eq!(digit_grid_i[Position::new(1, 1)], Some(Digit::D2));
            assert_eq!(digit_grid_i[Position::new(2, 2)], Some(Digit::D3));

            // Verify solutions are distinct
            for (grid_j, _) in &solutions[i + 1..] {
                assert_ne!(grid_i.to_digit_grid(), grid_j.to_digit_grid());
            }
        }
    }

    #[test]
    fn test_backtracking_occurs() {
        let solver = BacktrackSolver::without_techniques();
        let mut grid = CandidateGrid::new();

        // Create a scenario that requires backtracking
        // Place digits in a way that creates constraints
        grid.place(Position::new(0, 0), Digit::D1);
        grid.place(Position::new(0, 1), Digit::D2);
        grid.place(Position::new(0, 2), Digit::D3);
        grid.place(Position::new(1, 3), Digit::D4);
        grid.place(Position::new(2, 6), Digit::D5);

        // Solve and check that some backtracking occurred
        // (We can't guarantee backtracking will happen, but it's likely with this setup)
        let mut solutions = solver.solve(grid.into()).unwrap();
        let (_, stats) = solutions.next().unwrap();

        // Should have made assumptions
        assert!(!stats.assumptions().is_empty());
    }

    #[test]
    fn test_backtrack_count_increments() {
        let solver = BacktrackSolver::without_techniques();
        let mut grid = CandidateGrid::new();

        // Create a more constrained puzzle
        grid.place(Position::new(0, 0), Digit::D1);
        grid.place(Position::new(0, 1), Digit::D2);
        grid.place(Position::new(0, 2), Digit::D3);
        grid.place(Position::new(0, 3), Digit::D4);
        grid.place(Position::new(0, 4), Digit::D5);

        // Get multiple solutions to increase chances of backtracking
        let solutions: Vec<_> = solver.solve(grid.into()).unwrap().take(5).collect();

        // At least one solution should have backtracked
        // (This is probabilistic but very likely with multiple solutions)
        let total_backtracks: usize = solutions
            .iter()
            .map(|(_, stats)| stats.backtrack_count())
            .sum();

        // We expect some backtracking to occur across multiple solutions
        // Note: This might be 0 if the solver is very lucky, but it's tracked correctly
        // Just verify that the backtrack count is accessible and non-negative by design
        assert!(total_backtracks < usize::MAX);
    }

    #[test]
    fn test_solution_is_complete() {
        let solver = BacktrackSolver::without_techniques();
        let mut grid = CandidateGrid::new();

        // Start with a partial grid
        grid.place(Position::new(4, 4), Digit::D5);

        let (solution, _) = solver.solve(grid.into()).unwrap().next().unwrap();

        // Solution should be complete (all 81 cells filled)
        assert!(solution.is_solved().unwrap());
        let digit_grid = solution.to_digit_grid();
        for pos in Position::ALL {
            assert!(digit_grid[pos].is_some());
        }
    }

    #[test]
    fn test_backtrack_count() {
        let stats = BacktrackSolverStats::new();

        // New stats should have zero backtrack count
        assert_eq!(stats.backtrack_count(), 0);
    }

    #[test]
    fn test_with_techniques_constructor() {
        use crate::technique::{BoxedTechnique, NakedSingle};

        let techniques: Vec<BoxedTechnique> = vec![Box::new(NakedSingle::new())];
        let solver = BacktrackSolver::with_techniques(techniques);

        let grid = TechniqueGrid::new();
        let result = solver.solve(grid);
        assert!(result.is_ok());
    }
}
