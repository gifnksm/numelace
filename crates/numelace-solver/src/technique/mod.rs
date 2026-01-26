//! Sudoku solving techniques.
//!
//! This module provides various techniques for solving Sudoku puzzles.
//! Each technique implements the [`Technique`] trait and can be applied to a candidate grid.

use std::fmt::Debug;

use numelace_core::CandidateGrid;

pub use self::{hidden_single::HiddenSingle, naked_single::NakedSingle};
use crate::SolverError;

mod hidden_single;
mod naked_single;

/// Returns all available techniques.
///
/// Techniques are ordered from easiest to hardest.
/// This list may grow as new techniques are implemented.
#[must_use]
pub fn all_techniques() -> Vec<BoxedTechnique> {
    fundamental_techniques()
    // Future: add more advanced techniques here
}

/// Returns the fundamental techniques.
///
/// These are the most basic logical techniques for solving Sudoku puzzles:
/// - **Naked Single**: A cell with only one remaining candidate
/// - **Hidden Single**: A digit that can only go in one cell within a house
///
/// These techniques form the foundation of technique-based solving and are
/// essential for [`TechniqueSolver`](crate::TechniqueSolver). While more
/// advanced techniques can provide additional solving power, these Singles
/// techniques represent the core logical deductions that human solvers
/// typically apply first.
///
/// This set remains stable over time, serving as a consistent baseline for
/// benchmarking even as more advanced techniques are added to [`all_techniques`].
///
/// # Examples
///
/// ```
/// use numelace_solver::technique;
///
/// let techniques = technique::fundamental_techniques();
/// assert_eq!(techniques.len(), 2);
/// ```
///
/// # See Also
///
/// - [`all_techniques`] - Includes all available techniques (may grow over time)
#[must_use]
pub fn fundamental_techniques() -> Vec<BoxedTechnique> {
    vec![Box::new(NakedSingle::new()), Box::new(HiddenSingle::new())]
}

/// A trait representing a Sudoku solving technique.
///
/// Each technique is applied to a candidate grid and updates cell values or candidates.
pub trait Technique: Debug {
    /// Returns the name of the technique.
    fn name(&self) -> &'static str;

    /// Returns a boxed clone of the technique.
    fn clone_box(&self) -> BoxedTechnique;

    /// Applies the technique to a candidate grid.
    ///
    /// # Arguments
    ///
    /// * `grid` - The candidate grid
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - The technique was applied and the grid was updated
    /// * `Ok(false)` - The technique was applied but the grid was not updated
    ///
    /// # Errors
    ///
    /// Returns an error if the technique detects an invalid state in the grid.
    fn apply(&self, grid: &mut CandidateGrid) -> Result<bool, SolverError>;
}

/// A boxed technique.
pub type BoxedTechnique = Box<dyn Technique>;

impl Clone for BoxedTechnique {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
