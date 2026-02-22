//! Sudoku solving techniques.
//!
//! This module provides various techniques for solving Sudoku puzzles.
//! Each technique implements the [`Technique`] trait and can be applied to a [`TechniqueGrid`].
//!
//! [`Technique`]: crate::Technique
//! [`TechniqueGrid`]: crate::TechniqueGrid

pub use self::{
    hidden_pair::HiddenPair, hidden_quad::HiddenQuad, hidden_single::HiddenSingle,
    hidden_triple::HiddenTriple, locked_candidates::LockedCandidates, naked_pair::NakedPair,
    naked_quad::NakedQuad, naked_single::NakedSingle, naked_triple::NakedTriple, x_wing::XWing,
    y_wing::YWing,
};

mod hidden_pair;
mod hidden_quad;
mod hidden_single;
mod hidden_triple;
mod locked_candidates;
mod naked_pair;
mod naked_quad;
mod naked_single;
mod naked_triple;
pub(crate) mod traits;
mod x_wing;
mod y_wing;

/// Returns all available techniques.
///
/// Techniques are ordered from easiest to hardest.
/// This list may grow as new techniques are implemented.
#[must_use]
pub fn all_techniques() -> Vec<crate::BoxedTechnique> {
    upper_intermediate_techniques()
}

/// Returns the fundamental techniques.
///
/// This includes [`NakedSingle`] and [`HiddenSingle`], and stays stable as a
/// baseline set even when [`all_techniques`] grows.
#[must_use]
pub fn fundamental_techniques() -> Vec<crate::BoxedTechnique> {
    vec![Box::new(NakedSingle::new()), Box::new(HiddenSingle::new())]
}

/// Returns the basic techniques used by the solver.
///
/// This includes the fundamental Singles plus [`LockedCandidates`].
#[must_use]
pub fn basic_techniques() -> Vec<crate::BoxedTechnique> {
    let mut techniques = fundamental_techniques();
    techniques.push(Box::new(LockedCandidates::new()));
    techniques
}

/// Returns the intermediate techniques used by the solver.
///
/// This currently includes the basic techniques plus [`NakedPair`], [`HiddenPair`], [`NakedTriple`] and [`HiddenTriple`].
#[must_use]
pub fn intermediate_techniques() -> Vec<crate::BoxedTechnique> {
    let mut techniques = basic_techniques();
    techniques.push(Box::new(NakedPair::new()));
    techniques.push(Box::new(HiddenPair::new()));
    techniques.push(Box::new(NakedTriple::new()));
    techniques.push(Box::new(HiddenTriple::new()));
    techniques
}

/// Returns the upper-intermediate techniques used by the solver.
///
/// This currently includes the intermediate techniques plus [`NakedQuad`],
/// [`HiddenQuad`], [`XWing`], and [`YWing`]. Additional upper-intermediate
/// techniques may be appended here as they are implemented.
#[must_use]
pub fn upper_intermediate_techniques() -> Vec<crate::BoxedTechnique> {
    let mut techniques = intermediate_techniques();
    techniques.push(Box::new(NakedQuad::new()));
    techniques.push(Box::new(HiddenQuad::new()));
    techniques.push(Box::new(XWing::new()));
    techniques.push(Box::new(YWing::new()));
    // techniques.push(Box::new(Skyscraper::new()));
    // techniques.push(Box::new(TwoStringKite::new()));
    // techniques.push(Box::new(XChain::new()));
    techniques
}
