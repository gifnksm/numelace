//! Sudoku solving techniques.
//!
//! This module provides various techniques for solving Sudoku puzzles.
//! Each technique implements the [`Technique`] trait and can be applied to a [`TechniqueGrid`].
//!
//! [`Technique`]: crate::Technique
//! [`TechniqueGrid`]: crate::TechniqueGrid

pub use self::{
    hidden_pair::*, hidden_quad::*, hidden_single::*, hidden_triple::*, jellyfish::*,
    locked_candidates::*, naked_pair::*, naked_quad::*, naked_single::*, naked_triple::*,
    remote_pair::*, skyscraper::*, swordfish::*, two_string_kite::*, x_chain::*, x_wing::*,
    xy_chain::*, y_wing::*,
};
use crate::{BoxedTechnique, TechniqueTier};

mod hidden_pair;
mod hidden_quad;
mod hidden_single;
mod hidden_triple;
mod jellyfish;
mod locked_candidates;
mod naked_pair;
mod naked_quad;
mod naked_single;
mod naked_triple;
mod remote_pair;
mod skyscraper;
mod swordfish;
pub(crate) mod traits;
mod two_string_kite;
mod x_chain;
mod x_wing;
mod xy_chain;
mod y_wing;

/// Finds a technique by its stable ID.
#[must_use]
pub fn find_technique_by_id(id: &str) -> Option<BoxedTechnique> {
    all_techniques().into_iter().find(|tech| tech.id() == id)
}

/// Returns all available techniques, ordered from easiest to hardest.
#[must_use]
pub fn all_techniques() -> Vec<BoxedTechnique> {
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
        Box::new(TwoStringKite::new()),
        Box::new(YWing::new()),
        Box::new(Swordfish::new()),
        Box::new(Jellyfish::new()),
        Box::new(RemotePair::new()),
        Box::new(XChain::new()),
        Box::new(XyChain::new()),
    ]
}

/// Returns the fundamental techniques at or below the fundamental tier.
#[must_use]
pub fn fundamental_techniques() -> Vec<BoxedTechnique> {
    all_techniques()
        .into_iter()
        .filter(|tech| tech.tier() <= TechniqueTier::Fundamental)
        .collect()
}

/// Returns the basic techniques at or below the basic tier.
#[must_use]
pub fn basic_techniques() -> Vec<BoxedTechnique> {
    all_techniques()
        .into_iter()
        .filter(|tech| tech.tier() <= TechniqueTier::Basic)
        .collect()
}

/// Returns the intermediate techniques at or below the intermediate tier.
#[must_use]
pub fn intermediate_techniques() -> Vec<BoxedTechnique> {
    all_techniques()
        .into_iter()
        .filter(|tech| tech.tier() <= TechniqueTier::Intermediate)
        .collect()
}

/// Returns the upper-intermediate techniques at or below the upper-intermediate tier.
#[must_use]
pub fn upper_intermediate_techniques() -> Vec<BoxedTechnique> {
    all_techniques()
        .into_iter()
        .filter(|tech| tech.tier() <= TechniqueTier::UpperIntermediate)
        .collect()
}

/// Returns the advanced techniques at or below the advanced tier.
#[must_use]
pub fn advanced_techniques() -> Vec<BoxedTechnique> {
    all_techniques()
        .into_iter()
        .filter(|tech| tech.tier() <= TechniqueTier::Advanced)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technieues_sorted_by_tier() {
        for techniques in all_techniques().windows(2) {
            assert!(techniques[0].tier() <= techniques[1].tier());
        }
    }
}
