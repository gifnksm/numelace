use std::fmt::Debug;

use crate::{BoxedTechniqueStep, SolverError, TechniqueGrid};

/// A trait representing a Sudoku solving technique.
///
/// Each technique operates on a [`TechniqueGrid`] and updates cell values or candidates.
pub trait Technique: Debug + Send + Sync {
    /// Returns the name of the technique.
    fn name(&self) -> &'static str;

    /// Returns the technique tier used for difficulty ordering.
    fn tier(&self) -> TechniqueTier;

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

/// Difficulty tier used to order techniques from easiest to hardest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TechniqueTier {
    /// Fundamental techniques (baseline essentials).
    Fundamental,
    /// Basic techniques built on fundamental ones.
    Basic,
    /// Intermediate techniques for more complex deductions.
    Intermediate,
    /// Upper-intermediate techniques that go beyond standard intermediate.
    UpperIntermediate,
    /// Advanced techniques intended for harder puzzles.
    Advanced,
    /// Expert-level techniques for the toughest puzzles.
    Expert,
}

/// A boxed technique.
pub type BoxedTechnique = Box<dyn Technique>;

impl Clone for BoxedTechnique {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
