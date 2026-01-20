use std::fmt::Debug;

use sudoku_core::CandidateGrid;

use crate::SolverError;

mod hidden_single;
mod naked_single;

#[must_use]
pub fn all_techniques() -> Vec<BoxedTechnique> {
    vec![
        Box::new(naked_single::NakedSingle::new()),
        Box::new(hidden_single::HiddenSingle::new()),
    ]
}

pub trait Technique: Debug {
    fn name(&self) -> &str;
    fn clone_box(&self) -> BoxedTechnique;
    fn apply(&self, grid: &mut CandidateGrid) -> Result<bool, SolverError>;
}

pub type BoxedTechnique = Box<dyn Technique>;

impl Clone for BoxedTechnique {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
