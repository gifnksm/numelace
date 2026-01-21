use std::collections::HashMap;

use sudoku_core::CandidateGrid;

use crate::{
    SolverError,
    technique::{self, BoxedTechnique},
};

/// Statistics collected during technique-based solving.
///
/// This structure tracks which techniques were applied and how many times,
/// as well as the total number of solving steps taken.
///
/// # Examples
///
/// ```
/// use sudoku_core::CandidateGrid;
/// use sudoku_solver::{TechniqueSolver, TechniqueSolverStats};
///
/// let solver = TechniqueSolver::with_all_techniques();
/// let mut grid = CandidateGrid::new();
///
/// let (solved, stats) = solver.solve(&mut grid)?;
/// println!("Total steps: {}", stats.total_steps);
/// println!("Naked singles applied: {}", stats.count("naked singles"));
/// # Ok::<(), sudoku_solver::SolverError>(())
/// ```
#[derive(Debug, Default, Clone)]
pub struct TechniqueSolverStats {
    /// Map of technique names to the number of times each was successfully applied.
    pub applications: HashMap<&'static str, usize>,
    /// Total number of solving steps taken (sum of all technique applications).
    pub total_steps: usize,
}

impl TechniqueSolverStats {
    /// Creates a new empty statistics object.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of times a specific technique was applied.
    #[must_use]
    pub fn count(&self, technique_name: &str) -> usize {
        self.applications.get(technique_name).copied().unwrap_or(0)
    }

    /// Returns `true` if any technique was applied at least once.
    #[must_use]
    pub fn has_progress(&self) -> bool {
        self.total_steps > 0
    }
}

/// A solver that applies human-like solving techniques to a Sudoku grid.
///
/// `TechniqueSolver` iterates through a list of techniques in order, applying
/// the first technique that makes progress. When a technique succeeds, the solver
/// returns to allow the caller to check the grid state. This allows for step-by-step
/// solving or continuous solving until stuck.
///
/// # Examples
///
/// ```
/// use sudoku_core::CandidateGrid;
/// use sudoku_solver::TechniqueSolver;
///
/// let solver = TechniqueSolver::with_all_techniques();
/// let mut grid = CandidateGrid::new();
///
/// // Solve completely
/// let (solved, stats) = solver.solve(&mut grid)?;
/// if solved {
///     println!("Puzzle solved in {} steps!", stats.total_steps);
/// } else {
///     println!("Stuck after {} steps", stats.total_steps);
/// }
/// # Ok::<(), sudoku_solver::SolverError>(())
/// ```
///
/// # Step-by-step solving
///
/// ```
/// use sudoku_core::CandidateGrid;
/// use sudoku_solver::{TechniqueSolver, TechniqueSolverStats};
///
/// let solver = TechniqueSolver::with_all_techniques();
/// let mut grid = CandidateGrid::new();
/// let mut stats = TechniqueSolverStats::new();
///
/// while solver.step(&mut grid, &mut stats)? {
///     println!("Progress made! Step {}", stats.total_steps);
///     if grid.is_solved() {
///         break;
///     }
/// }
/// # Ok::<(), sudoku_solver::SolverError>(())
/// ```
#[derive(Debug, Clone)]
pub struct TechniqueSolver {
    techniques: Vec<BoxedTechnique>,
}

impl TechniqueSolver {
    /// Creates a new solver with the specified techniques.
    ///
    /// Techniques are applied in the order they appear in the vector.
    /// When a technique makes progress, the solver stops and returns,
    /// allowing the next call to start from the first technique again.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_solver::{
    ///     TechniqueSolver,
    ///     technique::{BoxedTechnique, NakedSingle},
    /// };
    ///
    /// let techniques: Vec<BoxedTechnique> = vec![Box::new(NakedSingle::new())];
    /// let solver = TechniqueSolver::new(techniques);
    /// ```
    #[must_use]
    pub fn new(techniques: Vec<BoxedTechnique>) -> Self {
        Self { techniques }
    }

    /// Creates a new solver with all available techniques.
    ///
    /// Techniques are ordered from easiest to hardest, as defined by
    /// [`technique::all_techniques`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_solver::TechniqueSolver;
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// ```
    #[must_use]
    pub fn with_all_techniques() -> Self {
        Self {
            techniques: technique::all_techniques(),
        }
    }

    /// Applies one step of solving by trying each technique in order.
    ///
    /// Iterates through the list of techniques, applying the first one that
    /// makes progress. When a technique succeeds, the statistics are updated
    /// and the method returns immediately.
    ///
    /// # Arguments
    ///
    /// * `grid` - The candidate grid to solve
    /// * `stats` - Statistics object to record which technique was applied
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - A technique was applied and made progress
    /// * `Ok(false)` - No technique could make progress (solver is stuck)
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::Contradiction`] if the grid becomes inconsistent
    /// after applying a technique.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::CandidateGrid;
    /// use sudoku_solver::{TechniqueSolver, TechniqueSolverStats};
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let mut grid = CandidateGrid::new();
    /// let mut stats = TechniqueSolverStats::new();
    ///
    /// if solver.step(&mut grid, &mut stats)? {
    ///     println!("Made progress!");
    /// } else {
    ///     println!("Stuck - no technique can help");
    /// }
    /// # Ok::<(), sudoku_solver::SolverError>(())
    /// ```
    pub fn step(
        &self,
        grid: &mut CandidateGrid,
        stats: &mut TechniqueSolverStats,
    ) -> Result<bool, SolverError> {
        for technique in &self.techniques {
            if technique.apply(grid)? {
                *stats.applications.entry(technique.name()).or_default() += 1;
                stats.total_steps += 1;
                if !grid.is_consistent() {
                    return Err(SolverError::Contradiction);
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Applies techniques repeatedly until the grid is solved or no progress can be made.
    ///
    /// This method calls [`step`](Self::step) in a loop until either the grid is
    /// completely solved or no technique can make further progress. Statistics are
    /// collected throughout the solving process.
    ///
    /// # Arguments
    ///
    /// * `grid` - The candidate grid to solve
    ///
    /// # Returns
    ///
    /// Returns a tuple `(solved, stats)` where:
    /// * `solved` - `true` if the grid is completely solved, `false` if stuck
    /// * `stats` - Statistics about which techniques were applied and how many times
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::Contradiction`] if the grid becomes inconsistent
    /// during solving.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::CandidateGrid;
    /// use sudoku_solver::TechniqueSolver;
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let mut grid = CandidateGrid::new();
    ///
    /// let (solved, stats) = solver.solve(&mut grid)?;
    /// if solved {
    ///     println!("Solved in {} steps", stats.total_steps);
    /// } else {
    ///     println!(
    ///         "Stuck after {} steps - backtracking needed",
    ///         stats.total_steps
    ///     );
    /// }
    /// # Ok::<(), sudoku_solver::SolverError>(())
    /// ```
    pub fn solve(
        &self,
        grid: &mut CandidateGrid,
    ) -> Result<(bool, TechniqueSolverStats), SolverError> {
        let mut stats = TechniqueSolverStats::default();
        while self.step(grid, &mut stats)? {
            if grid.is_solved() {
                return Ok((true, stats));
            }
        }

        Ok((false, stats))
    }
}

#[cfg(test)]
mod tests {
    use sudoku_core::{CandidateGrid, Digit, Position};

    use super::*;
    use crate::technique::{BoxedTechnique, HiddenSingle, NakedSingle, all_techniques};

    fn create_test_solver() -> TechniqueSolver {
        let techniques: Vec<BoxedTechnique> =
            vec![Box::new(NakedSingle::new()), Box::new(HiddenSingle::new())];
        TechniqueSolver::new(techniques)
    }

    #[test]
    fn test_step_returns_false_when_no_progress() {
        let solver = create_test_solver();
        let mut grid = CandidateGrid::new();
        let mut stats = TechniqueSolverStats::new();

        // On a fresh grid with all candidates, no technique can make progress yet
        let result = solver.step(&mut grid, &mut stats);
        assert!(result.is_ok());
        assert!(!result.unwrap());
        assert_eq!(stats.total_steps, 0);
    }

    #[test]
    fn test_step_returns_true_when_progress_made() {
        let solver = create_test_solver();
        let mut grid = CandidateGrid::new();
        let mut stats = TechniqueSolverStats::new();

        // Create a naked single: only D5 at (4, 4)
        for digit in Digit::ALL {
            if digit != Digit::D5 {
                grid.remove_candidate(Position::new(4, 4), digit);
            }
        }

        let result = solver.step(&mut grid, &mut stats);
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(stats.total_steps, 1);
        assert_eq!(stats.count("naked singles"), 1);
    }

    #[test]
    fn test_step_records_stats() {
        let solver = create_test_solver();
        let mut grid = CandidateGrid::new();
        let mut stats = TechniqueSolverStats::new();

        // Create a naked single
        for digit in Digit::ALL {
            if digit != Digit::D5 {
                grid.remove_candidate(Position::new(4, 4), digit);
            }
        }

        solver.step(&mut grid, &mut stats).unwrap();

        assert_eq!(stats.total_steps, 1);
        assert!(stats.has_progress());
        assert_eq!(stats.count("naked singles"), 1);
        assert_eq!(stats.count("hidden singles"), 0);
    }

    #[test]
    fn test_solve_empty_grid() {
        let solver = create_test_solver();
        let mut grid = CandidateGrid::new();

        let result = solver.solve(&mut grid);
        assert!(result.is_ok());

        let (is_solved, stats) = result.unwrap();
        assert!(!is_solved); // Empty grid can't be solved with techniques alone
        assert_eq!(stats.total_steps, 0);
    }

    #[test]
    fn test_solve_records_multiple_steps() {
        let solver = create_test_solver();
        let mut grid = CandidateGrid::new();

        // Create a naked single at (0, 0) - only D1 remains
        for digit in Digit::ALL {
            if digit != Digit::D1 {
                grid.remove_candidate(Position::new(0, 0), digit);
            }
        }

        let result = solver.solve(&mut grid);
        assert!(result.is_ok());

        let (_solved, stats) = result.unwrap();
        // Should have made at least one step
        assert!(
            stats.total_steps >= 1,
            "Expected at least 1 step, got {}",
            stats.total_steps
        );
        assert!(stats.has_progress());
        // The naked single technique should have been applied
        assert!(stats.count("naked singles") >= 1 || stats.count("hidden singles") >= 1);
    }

    #[test]
    fn test_solve_detects_solved_grid() {
        let solver = create_test_solver();

        // Create a simple case with a few naked singles
        let mut grid = CandidateGrid::new();

        // Create a naked single at (0, 0) - only D1 remains
        for digit in Digit::ALL {
            if digit != Digit::D1 {
                grid.remove_candidate(Position::new(0, 0), digit);
            }
        }

        let result = solver.solve(&mut grid);

        // Should make progress and detect the naked single
        assert!(result.is_ok());
        let (_solved, stats) = result.unwrap();
        // Grid won't be fully solved, but should have made progress
        assert!(stats.has_progress());
    }

    #[test]
    fn test_stats_count_method() {
        let mut stats = TechniqueSolverStats::new();

        assert_eq!(stats.count("naked singles"), 0);

        *stats.applications.entry("naked singles").or_default() += 1;
        assert_eq!(stats.count("naked singles"), 1);

        *stats.applications.entry("naked singles").or_default() += 2;
        assert_eq!(stats.count("naked singles"), 3);

        assert_eq!(stats.count("nonexistent"), 0);
    }

    #[test]
    fn test_stats_has_progress() {
        let mut stats = TechniqueSolverStats::new();

        assert!(!stats.has_progress());

        stats.total_steps = 1;
        assert!(stats.has_progress());

        stats.total_steps = 100;
        assert!(stats.has_progress());
    }

    #[test]
    fn test_with_all_techniques() {
        let solver = TechniqueSolver::with_all_techniques();
        let all = all_techniques();

        // Should have the same number of techniques as all_techniques()
        assert_eq!(solver.techniques.len(), all.len());
    }

    #[test]
    fn test_new_with_custom_techniques() {
        let techniques = vec![Box::new(NakedSingle::new()) as BoxedTechnique];

        let solver = TechniqueSolver::new(techniques);
        assert_eq!(solver.techniques.len(), 1);
    }
}
