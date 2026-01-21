//! Sudoku solving techniques.
//!
//! This module provides various techniques for solving Sudoku puzzles.
//! Each technique implements the [`Technique`] trait and can be applied to a candidate grid.

use std::fmt::Debug;

use sudoku_core::CandidateGrid;

pub use self::{hidden_single::HiddenSingle, naked_single::NakedSingle};
use crate::SolverError;

mod hidden_single;
mod naked_single;

/// Returns all available techniques.
///
/// Techniques are ordered from easiest to hardest.
#[must_use]
pub fn all_techniques() -> Vec<BoxedTechnique> {
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
