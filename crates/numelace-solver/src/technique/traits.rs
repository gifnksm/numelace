use std::fmt::Debug;

use crate::{BoxedTechniqueStep, SolverError, TechniqueApplication, TechniqueGrid};

/// A trait representing a Sudoku solving technique.
///
/// Each technique operates on a [`TechniqueGrid`] and updates cell values or candidates.
pub trait Technique: Debug + Send + Sync {
    /// Returns the ID of the technique.
    fn id(&self) -> &'static str;

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

    /// Finds all steps produced by this technique in a single pass without mutating the grid.
    ///
    /// This repeats [`Technique::find_step`] against a cloned grid, applying each step's
    /// [`TechniqueApplication`] to advance the pass until no further steps are found.
    ///
    /// # Returns
    ///
    /// Returns the ordered list of steps discovered during the pass.
    ///
    /// # Errors
    ///
    /// Returns an error if the technique detects an invalid state in the grid.
    fn find_pass(&self, grid: &TechniqueGrid) -> Result<Vec<BoxedTechniqueStep>, SolverError> {
        let mut steps = Vec::new();
        let mut current_grid = grid.clone();
        while let Some(step) = self.find_step(&current_grid)? {
            for app in step.application() {
                match app {
                    TechniqueApplication::Placement { position, digit } => {
                        current_grid.place(position, digit);
                    }
                    TechniqueApplication::CandidateElimination { positions, digits } => {
                        current_grid.remove_candidate_set_with_mask(positions, digits);
                    }
                }
            }
            current_grid.check_consistency()?;
            steps.push(step);
        }
        Ok(steps)
    }

    /// Applies a single technique step.
    ///
    /// Implementations should apply at most one logical step and return `Ok(true)`
    /// when any progress is made.
    ///
    /// # Errors
    ///
    /// Returns an error if the technique detects an invalid state in the grid.
    fn apply_step(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError>;

    /// Applies the technique across the grid in a single pass.
    ///
    /// This performs a full sweep for the technique, potentially applying multiple
    /// steps in one call. It does not loop until stuck.
    ///
    /// # Returns
    ///
    /// * `Ok(n)` - The number of applications performed (0 means no progress)
    ///
    /// # Errors
    ///
    /// Returns an error if the technique detects an invalid state in the grid.
    fn apply_pass(&self, grid: &mut TechniqueGrid) -> Result<usize, SolverError>;
}

/// Difficulty tier used to order techniques from easiest to hardest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::IsVariant)]
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
