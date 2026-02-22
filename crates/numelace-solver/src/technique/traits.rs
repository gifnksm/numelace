use std::fmt::Debug;

use crate::{BoxedTechniqueStep, SolverError, TechniqueGrid};

/// A trait representing a Sudoku solving technique.
///
/// Each technique operates on a [`TechniqueGrid`] and updates cell values or candidates.
pub trait Technique: Debug + Send + Sync {
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

/// A boxed technique.
pub type BoxedTechnique = Box<dyn Technique>;

impl Clone for BoxedTechnique {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
