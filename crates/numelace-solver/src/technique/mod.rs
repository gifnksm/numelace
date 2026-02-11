//! Sudoku solving techniques.
//!
//! This module provides various techniques for solving Sudoku puzzles.
//! Each technique implements the [`Technique`] trait and can be applied to a [`TechniqueGrid`].

use std::fmt::Debug;

use numelace_core::{
    CandidateGrid, ConsistencyError, Digit, DigitGrid, DigitPositions, DigitSet, Position,
};

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

/// Solver state for technique-based solving.
///
/// This is a lightweight wrapper around a [`CandidateGrid`] with technique
/// solver bookkeeping.
#[derive(Debug, Clone)]
pub struct TechniqueGrid {
    /// Underlying candidate state.
    candidates: CandidateGrid,
    /// Decided cells that have already had their peer eliminations applied.
    decided_propagated: DigitPositions,
}

impl From<DigitGrid> for TechniqueGrid {
    fn from(grid: DigitGrid) -> Self {
        CandidateGrid::from(grid).into()
    }
}

impl From<CandidateGrid> for TechniqueGrid {
    fn from(candidates: CandidateGrid) -> Self {
        Self {
            candidates,
            decided_propagated: DigitPositions::EMPTY,
        }
    }
}

impl Default for TechniqueGrid {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TechniqueGrid {
    /// Creates an empty technique grid with all candidates available.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::from(CandidateGrid::new())
    }

    /// Builds a technique grid from a digit grid.
    #[inline]
    #[must_use]
    pub fn from_digit_grid(grid: &DigitGrid) -> Self {
        Self::from(CandidateGrid::from_digit_grid(grid))
    }

    /// Returns the underlying candidate grid.
    #[inline]
    #[must_use]
    pub fn candidates(&self) -> &CandidateGrid {
        &self.candidates
    }

    /// Returns a mutable reference to the underlying candidate grid.
    #[inline]
    #[must_use]
    pub fn candidates_mut(&mut self) -> &mut CandidateGrid {
        &mut self.candidates
    }

    /// Consumes the wrapper and returns the underlying candidate grid.
    #[inline]
    #[must_use]
    pub fn into_candidates(self) -> CandidateGrid {
        self.candidates
    }

    /// Returns a digit grid containing only decided cells.
    #[inline]
    #[must_use]
    pub fn to_digit_grid(&self) -> DigitGrid {
        self.candidates.to_digit_grid()
    }

    /// Checks whether the candidate grid is consistent.
    ///
    /// # Errors
    ///
    /// Returns [`ConsistencyError`] if the grid contains contradictions.
    #[inline]
    pub fn check_consistency(&self) -> Result<(), ConsistencyError> {
        self.candidates.check_consistency()
    }

    /// Returns whether the candidate grid is fully solved.
    ///
    /// # Errors
    ///
    /// Returns [`ConsistencyError`] if the grid contains contradictions.
    #[inline]
    pub fn is_solved(&self) -> Result<bool, ConsistencyError> {
        self.candidates.is_solved()
    }
}

/// A trait representing a Sudoku solving technique.
///
/// Each technique operates on a [`TechniqueGrid`] and updates cell values or candidates.
pub trait Technique: Debug {
    /// Returns the name of the technique.
    fn name(&self) -> &'static str;

    /// Returns a boxed clone of the technique.
    fn clone_box(&self) -> BoxedTechnique;

    /// Finds the next hint step without mutating the grid.
    ///
    /// Returns `Ok(None)` when this technique has no applicable step.
    ///
    /// # Errors
    ///
    /// Returns an error if the technique detects an invalid state in the grid.
    fn find_step(&self, grid: &TechniqueGrid) -> Result<Option<BoxedTechniqueStep>, SolverError>;

    /// Applies the technique to a technique grid.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - The technique was applied and the grid was updated
    /// * `Ok(false)` - The technique was applied but the grid was not updated
    ///
    /// # Errors
    ///
    /// Returns an error if the technique detects an invalid state in the grid.
    fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError>;
}

/// Cells involved in a technique's applicability conditions.
pub type ConditionCells = DigitPositions;

/// Pairs of (cells, digits) involved in a technique's applicability conditions.
pub type ConditionDigitCells = Vec<(DigitPositions, DigitSet)>;

/// A hint step produced by a technique.
pub trait TechniqueStep: Debug {
    /// Returns the name of the technique that produced this step.
    fn technique_name(&self) -> &'static str;

    /// Returns a boxed clone of the step.
    fn clone_box(&self) -> BoxedTechniqueStep;

    /// Returns the cells involved in the applicability conditions.
    ///
    /// These are the cells that justify applying the technique. Hint systems may
    /// use this to highlight relevant cells before naming the technique.
    fn condition_cells(&self) -> ConditionCells;

    /// Returns condition pairs of (cells, digits) involved in applicability.
    ///
    /// Each pair provides a set of cells and the digits that matter for the
    /// technique's conditions. Hint systems may use this as a more detailed
    /// explanation of the underlying logic.
    fn condition_digit_cells(&self) -> ConditionDigitCells;

    /// Returns the concrete changes produced by applying the technique.
    fn application(&self) -> Vec<TechniqueApplication>;
}

/// Concrete changes produced by applying a technique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechniqueApplication {
    /// Place a digit in a single cell.
    Placement {
        /// Cell to place the digit into.
        position: Position,
        /// Digit to place.
        digit: Digit,
    },
    /// Remove candidates from the specified positions.
    CandidateElimination {
        /// Positions where candidates are removed.
        positions: DigitPositions,
        /// Digits to remove from the specified positions.
        digits: DigitSet,
    },
}

/// A boxed technique.
pub type BoxedTechnique = Box<dyn Technique>;

impl Clone for BoxedTechnique {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// A boxed technique step.
pub type BoxedTechniqueStep = Box<dyn TechniqueStep>;

impl Clone for BoxedTechniqueStep {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
