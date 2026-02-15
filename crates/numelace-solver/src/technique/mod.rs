//! Sudoku solving techniques.
//!
//! This module provides various techniques for solving Sudoku puzzles.
//! Each technique implements the [`Technique`] trait and can be applied to a [`TechniqueGrid`].

use std::fmt::Debug;

use numelace_core::{Digit, DigitPositions, DigitSet, Position};

pub use self::{
    hidden_pair::HiddenPair, hidden_single::HiddenSingle, locked_candidates::LockedCandidates,
    naked_pair::NakedPair, naked_single::NakedSingle,
};
use crate::{SolverError, TechniqueGrid};

mod hidden_pair;
mod hidden_single;
mod locked_candidates;
mod naked_pair;
mod naked_single;

/// Returns all available techniques.
///
/// Techniques are ordered from easiest to hardest.
/// This list may grow as new techniques are implemented.
#[must_use]
pub fn all_techniques() -> Vec<BoxedTechnique> {
    intermediate_techniques()
}

/// Returns the fundamental techniques.
///
/// This includes [`NakedSingle`] and [`HiddenSingle`], and stays stable as a
/// baseline set even when [`all_techniques`] grows.
#[must_use]
pub fn fundamental_techniques() -> Vec<BoxedTechnique> {
    vec![Box::new(NakedSingle::new()), Box::new(HiddenSingle::new())]
}

/// Returns the basic techniques used by the solver.
///
/// This includes the fundamental Singles plus [`LockedCandidates`].
#[must_use]
pub fn basic_techniques() -> Vec<BoxedTechnique> {
    let mut techniques = fundamental_techniques();
    techniques.push(Box::new(LockedCandidates::new()));
    techniques
}

/// Returns the intermediate techniques used by the solver.
///
/// This currently includes the basic techniques plus [`NakedPair`] and [`HiddenPair`].
#[must_use]
pub fn intermediate_techniques() -> Vec<BoxedTechnique> {
    let mut techniques = basic_techniques();
    techniques.push(Box::new(NakedPair::new()));
    techniques.push(Box::new(HiddenPair::new()));
    // techniques.push(Box::new(NakedTriple::new()));
    // techniques.push(Box::new(HiddenTriple::new()));
    techniques
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
