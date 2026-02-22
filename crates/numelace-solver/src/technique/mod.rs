//! Sudoku solving techniques.
//!
//! This module provides various techniques for solving Sudoku puzzles.
//! Each technique implements the [`Technique`] trait and can be applied to a [`TechniqueGrid`].
//!
//! [`Technique`]: crate::Technique
//! [`TechniqueGrid`]: crate::TechniqueGrid

pub use self::{
    hidden_pair::*, hidden_quad::*, hidden_single::*, hidden_triple::*, locked_candidates::*,
    naked_pair::*, naked_quad::*, naked_single::*, naked_triple::*, skyscraper::*, x_wing::*,
    y_wing::*,
};
use crate::TechniqueTier;

mod hidden_pair;
mod hidden_quad;
mod hidden_single;
mod hidden_triple;
mod locked_candidates;
mod naked_pair;
mod naked_quad;
mod naked_single;
mod naked_triple;
mod skyscraper;
pub(crate) mod traits;
mod x_wing;
mod y_wing;

/// Returns all available techniques.
///
/// Techniques are ordered from easiest to hardest.
/// This list may grow as new techniques are implemented.
#[must_use]
pub fn all_techniques() -> Vec<crate::BoxedTechnique> {
    vec![
        Box::new(NakedSingle::new()),
        Box::new(HiddenSingle::new()),
        Box::new(LockedCandidates::new()),
        Box::new(NakedPair::new()),
        Box::new(HiddenPair::new()),
        Box::new(NakedTriple::new()),
        Box::new(HiddenTriple::new()),
        Box::new(NakedQuad::new()),
        Box::new(HiddenQuad::new()),
        Box::new(XWing::new()),
        Box::new(Skyscraper::new()),
        // Box::new(TwoStringKite::new()),
        Box::new(YWing::new()),
        // Box::new(XChain::new()),
    ]
}

/// Returns the fundamental techniques.
///
/// This includes techniques at or below the fundamental tier, and stays stable
/// as a baseline set even when [`all_techniques`] grows.
#[must_use]
pub fn fundamental_techniques() -> Vec<crate::BoxedTechnique> {
    all_techniques()
        .into_iter()
        .filter(|tech| tech.tier() <= TechniqueTier::Fundamental)
        .collect()
}

/// Returns the basic techniques used by the solver.
///
/// This includes techniques at or below the basic tier.
#[must_use]
pub fn basic_techniques() -> Vec<crate::BoxedTechnique> {
    all_techniques()
        .into_iter()
        .filter(|tech| tech.tier() <= TechniqueTier::Basic)
        .collect()
}

/// Returns the intermediate techniques used by the solver.
///
/// This includes techniques at or below the intermediate tier.
#[must_use]
pub fn intermediate_techniques() -> Vec<crate::BoxedTechnique> {
    all_techniques()
        .into_iter()
        .filter(|tech| tech.tier() <= TechniqueTier::Intermediate)
        .collect()
}

/// Returns the upper-intermediate techniques used by the solver.
///
/// This includes techniques at or below the upper-intermediate tier. Additional
/// techniques may be appended here as they are implemented.
#[must_use]
pub fn upper_intermediate_techniques() -> Vec<crate::BoxedTechnique> {
    all_techniques()
        .into_iter()
        .filter(|tech| tech.tier() <= TechniqueTier::UpperIntermediate)
        .collect()
}
