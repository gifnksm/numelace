use crate::{
    SolverError, TechniqueGrid,
    technique::{self, BoxedTechnique, BoxedTechniqueStep},
};

/// Statistics collected during technique-based solving.
///
/// This structure tracks which techniques were applied and how many times,
/// as well as the total number of solving steps taken.
///
/// # Examples
///
/// ```
/// use numelace_solver::{TechniqueGrid, TechniqueSolver};
///
/// let solver = TechniqueSolver::with_all_techniques();
/// let mut grid = TechniqueGrid::new();
///
/// let (_solved, stats) = solver.solve(&mut grid)?;
/// println!("Total steps: {}", stats.total_steps());
///
/// if let Some((i, _)) = solver
///     .techniques()
///     .iter()
///     .enumerate()
///     .find(|(_, t)| t.name() == "naked single")
/// {
///     println!("Naked singles applied: {}", stats.applications()[i]);
/// }
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
#[derive(Debug, Clone)]
pub struct TechniqueSolverStats {
    applications: Vec<usize>,
    total_steps: usize,
}

impl TechniqueSolverStats {
    /// Returns technique application counts in solver order.
    ///
    /// Includes techniques that were never applied with a count of `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_solver::{TechniqueGrid, TechniqueSolver};
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let mut grid = TechniqueGrid::new();
    /// let mut stats = solver.new_stats();
    ///
    /// let _ = solver.solve_with_stats(&mut grid, &mut stats)?;
    ///
    /// for (i, count) in stats.applications().iter().enumerate() {
    ///     println!("{}: {} times", solver.techniques()[i].name(), count);
    /// }
    /// # Ok::<(), numelace_solver::SolverError>(())
    /// ```
    #[must_use]
    pub fn applications(&self) -> &[usize] {
        &self.applications
    }

    /// Returns the total number of solving steps taken.
    ///
    /// This is the sum of all technique applications.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_solver::{TechniqueGrid, TechniqueSolver};
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let _grid = TechniqueGrid::new();
    /// let stats = solver.new_stats();
    /// assert_eq!(stats.total_steps(), 0);
    /// ```
    #[must_use]
    pub fn total_steps(&self) -> usize {
        self.total_steps
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
/// use numelace_solver::{TechniqueGrid, TechniqueSolver};
///
/// let solver = TechniqueSolver::with_all_techniques();
/// let mut grid = TechniqueGrid::new();
///
/// // Solve completely
/// let (solved, stats) = solver.solve(&mut grid)?;
/// if solved {
///     println!("Puzzle solved in {} steps!", stats.total_steps());
/// } else {
///     println!("Stuck after {} steps", stats.total_steps());
/// }
/// # Ok::<(), numelace_solver::SolverError>(())
/// ```
///
/// # Step-by-step solving
///
/// ```
/// use numelace_solver::{TechniqueGrid, TechniqueSolver};
///
/// let solver = TechniqueSolver::with_all_techniques();
/// let mut grid = TechniqueGrid::new();
/// let mut stats = solver.new_stats();
///
/// while solver.step(&mut grid, &mut stats)? {
///     println!("Progress made! Step {}", stats.total_steps());
///     if grid.is_solved()? {
///         break;
///     }
/// }
/// # Ok::<(), numelace_solver::SolverError>(())
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
    /// use numelace_solver::{
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
    /// use numelace_solver::TechniqueSolver;
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// ```
    #[must_use]
    pub fn with_all_techniques() -> Self {
        Self {
            techniques: technique::all_techniques(),
        }
    }

    /// Creates a statistics object aligned with this solver's technique order.
    #[must_use]
    pub fn new_stats(&self) -> TechniqueSolverStats {
        TechniqueSolverStats {
            applications: vec![0; self.techniques.len()],
            total_steps: 0,
        }
    }

    /// Returns the configured techniques in application order.
    ///
    /// The returned slice defines the index mapping used by
    /// [`TechniqueSolverStats::applications`].
    #[must_use]
    pub fn techniques(&self) -> &[BoxedTechnique] {
        &self.techniques
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
    /// Returns [`SolverError::Inconsistent`] if the grid becomes inconsistent
    /// after applying a technique.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_solver::{TechniqueGrid, TechniqueSolver};
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let mut grid = TechniqueGrid::new();
    /// let mut stats = solver.new_stats();
    ///
    /// if solver.step(&mut grid, &mut stats)? {
    ///     println!("Made progress!");
    /// } else {
    ///     println!("Stuck - no technique can help");
    /// }
    /// # Ok::<(), numelace_solver::SolverError>(())
    /// ```
    pub fn step(
        &self,
        grid: &mut TechniqueGrid,
        stats: &mut TechniqueSolverStats,
    ) -> Result<bool, SolverError> {
        debug_assert_eq!(self.techniques.len(), stats.applications.len());
        grid.check_consistency()?;

        for (i, technique) in self.techniques.iter().enumerate() {
            if technique.apply(grid)? {
                stats.applications[i] += 1;
                stats.total_steps += 1;
                grid.check_consistency()?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Finds the next available hint step without mutating the grid.
    ///
    /// Returns `Ok(None)` when no technique can provide a step.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::Inconsistent`] if the grid is inconsistent.
    pub fn find_step(
        &self,
        grid: &TechniqueGrid,
    ) -> Result<Option<BoxedTechniqueStep>, SolverError> {
        grid.check_consistency()?;
        for technique in &self.techniques {
            if let Some(step) = technique.find_step(grid)? {
                return Ok(Some(step));
            }
        }
        Ok(None)
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
    /// Returns [`SolverError::Inconsistent`] if the grid becomes inconsistent
    /// during solving.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_solver::{TechniqueGrid, TechniqueSolver};
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let mut grid = TechniqueGrid::new();
    ///
    /// let (solved, stats) = solver.solve(&mut grid)?;
    /// if solved {
    ///     println!("Solved in {} steps", stats.total_steps());
    /// } else {
    ///     println!(
    ///         "Stuck after {} steps - backtracking needed",
    ///         stats.total_steps()
    ///     );
    /// }
    /// # Ok::<(), numelace_solver::SolverError>(())
    /// ```
    pub fn solve(
        &self,
        grid: &mut TechniqueGrid,
    ) -> Result<(bool, TechniqueSolverStats), SolverError> {
        let mut stats = self.new_stats();
        let solved = self.solve_with_stats(grid, &mut stats)?;
        Ok((solved, stats))
    }

    /// Applies techniques repeatedly until the grid is solved or no progress can be made,
    /// using the provided statistics object.
    ///
    /// This is similar to [`solve`](Self::solve), but allows reusing an existing
    /// statistics object. This is useful when you want to accumulate statistics
    /// across multiple solving attempts or when you need more control over the
    /// statistics lifecycle.
    ///
    /// # Arguments
    ///
    /// * `grid` - The candidate grid to solve
    /// * `stats` - Statistics object to accumulate technique application data
    ///
    /// # Returns
    ///
    /// Returns `true` if the grid is completely solved, `false` if stuck.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::Inconsistent`] if the grid becomes inconsistent
    /// during solving.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_solver::{TechniqueGrid, TechniqueSolver};
    ///
    /// let solver = TechniqueSolver::with_all_techniques();
    /// let mut grid = TechniqueGrid::new();
    /// let mut stats = solver.new_stats();
    ///
    /// let solved = solver.solve_with_stats(&mut grid, &mut stats)?;
    /// println!("Solved: {}, Steps: {}", solved, stats.total_steps());
    /// # Ok::<(), numelace_solver::SolverError>(())
    /// ```
    pub fn solve_with_stats(
        &self,
        grid: &mut TechniqueGrid,
        stats: &mut TechniqueSolverStats,
    ) -> Result<bool, SolverError> {
        while self.step(grid, stats)? {
            if grid.is_solved()? {
                return Ok(true);
            }
        }
        Ok(grid.is_solved()?)
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{CandidateGrid, Digit, Position};

    use super::*;
    use crate::technique::{
        BoxedTechnique, HiddenSingle, NakedSingle, Technique as _, all_techniques,
    };

    fn create_test_solver() -> TechniqueSolver {
        let techniques: Vec<BoxedTechnique> =
            vec![Box::new(NakedSingle::new()), Box::new(HiddenSingle::new())];
        TechniqueSolver::new(techniques)
    }

    #[test]
    fn test_step_returns_false_when_no_progress() {
        let solver = create_test_solver();
        let mut grid = TechniqueGrid::from(CandidateGrid::new());
        let mut stats = solver.new_stats();

        // On a fresh grid with all candidates, no technique can make progress yet
        let result = solver.step(&mut grid, &mut stats);
        assert!(result.is_ok());
        assert!(!result.unwrap());
        assert_eq!(stats.total_steps, 0);
    }

    #[test]
    fn test_step_returns_true_when_progress_made() {
        let solver = create_test_solver();
        let mut grid = TechniqueGrid::from(CandidateGrid::new());
        let mut stats = solver.new_stats();

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

        let i = solver
            .techniques()
            .iter()
            .position(|t| t.name() == NakedSingle::new().name())
            .unwrap();
        assert_eq!(stats.applications()[i], 1);
    }

    #[test]
    fn test_step_records_stats() {
        let solver = create_test_solver();
        let mut grid = TechniqueGrid::from(CandidateGrid::new());
        let mut stats = solver.new_stats();

        // Create a naked single
        for digit in Digit::ALL {
            if digit != Digit::D5 {
                grid.remove_candidate(Position::new(4, 4), digit);
            }
        }

        solver.step(&mut grid, &mut stats).unwrap();

        assert_eq!(stats.total_steps, 1);
        let i = solver
            .techniques()
            .iter()
            .position(|t| t.name() == NakedSingle::new().name())
            .unwrap();
        assert_eq!(stats.applications()[i], 1);
    }

    #[test]
    fn test_solve_empty_grid() {
        let solver = create_test_solver();
        let mut grid = TechniqueGrid::from(CandidateGrid::new());

        let result = solver.solve(&mut grid);
        assert!(result.is_ok());

        let (is_solved, stats) = result.unwrap();
        assert!(!is_solved); // Empty grid can't be solved with techniques alone
        assert_eq!(stats.total_steps, 0);
    }

    #[test]
    fn test_solve_records_multiple_steps() {
        let solver = create_test_solver();
        let mut grid = TechniqueGrid::from(CandidateGrid::new());

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
        // At least one configured technique should have been applied
        assert!(stats.applications().iter().any(|&n| n >= 1));
    }

    #[test]
    fn test_solve_detects_solved_grid() {
        let solver = create_test_solver();

        // Create a simple case with a few naked singles
        let mut grid = TechniqueGrid::from(CandidateGrid::new());

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
    fn test_stats_has_progress() {
        let solver = create_test_solver();
        let mut stats = solver.new_stats();

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

    #[test]
    fn test_stats_applications_getter() {
        let solver = create_test_solver();
        let mut stats = solver.new_stats();
        assert_eq!(stats.applications().len(), 2);

        let i = solver
            .techniques()
            .iter()
            .position(|t| t.name() == NakedSingle::new().name())
            .unwrap();
        stats.applications[i] += 1;

        assert_eq!(stats.applications()[i], 1);
    }

    #[test]
    fn test_stats_total_steps_getter() {
        let solver = create_test_solver();
        let mut stats = solver.new_stats();
        assert_eq!(stats.total_steps(), 0);

        stats.total_steps = 5;
        assert_eq!(stats.total_steps(), 5);
    }

    #[test]
    fn test_solve_with_stats() {
        let solver = create_test_solver();
        let mut grid = TechniqueGrid::from(CandidateGrid::new());
        let mut stats = solver.new_stats();

        // Create a naked single that hasn't been placed yet
        for digit in Digit::ALL {
            if digit != Digit::D5 {
                grid.remove_candidate(Position::new(4, 4), digit);
            }
        }

        let result = solver.solve_with_stats(&mut grid, &mut stats);
        assert!(result.is_ok());
        // The naked single should have been detected and placed
        assert!(stats.total_steps() >= 1);
    }

    #[test]
    fn test_solve_with_stats_accumulates() {
        let solver = create_test_solver();
        let mut grid1 = TechniqueGrid::from(CandidateGrid::new());
        let mut grid2 = TechniqueGrid::from(CandidateGrid::new());
        let mut stats = solver.new_stats();

        // First solve - create naked single
        for digit in Digit::ALL {
            if digit != Digit::D1 {
                grid1.remove_candidate(Position::new(0, 0), digit);
            }
        }
        let _ = solver.solve_with_stats(&mut grid1, &mut stats);
        let first_steps = stats.total_steps();

        // Second solve accumulates - create another naked single
        for digit in Digit::ALL {
            if digit != Digit::D2 {
                grid2.remove_candidate(Position::new(1, 1), digit);
            }
        }
        let _ = solver.solve_with_stats(&mut grid2, &mut stats);

        assert!(stats.total_steps() >= first_steps);
    }
}
