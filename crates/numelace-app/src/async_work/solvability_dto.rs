//! DTOs for solvability checks and candidate grid conversion helpers.
//!
//! This module provides compact, serializable representations of candidate grids
//! so they can be sent across worker boundaries. It also includes a solvability
//! result DTO that mirrors the app-level solvability states.

use serde::{Deserialize, Serialize};

use numelace_core::{CandidateGrid, Digit, DigitSet, Position};
use numelace_solver::BacktrackSolverStats;

/// DTO containing two candidate grids for solvability checks.
///
/// The flow first checks `with_user_notes`, and if that is inconsistent or has
/// no solution, it falls back to `without_user_notes`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SolvabilityRequestDto {
    pub(crate) with_user_notes: SolvabilityGridDto,
    pub(crate) without_user_notes: SolvabilityGridDto,
}

/// DTO representing solvability results across worker boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum SolvabilityStateDto {
    Inconsistent,
    NoSolution,
    Solvable {
        with_user_notes: bool,
        stats: SolvabilityStatsDto,
    },
}

/// Compact solver statistics used by solvability results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SolvabilityStatsDto {
    pub(crate) assumptions_len: usize,
    pub(crate) backtrack_count: usize,
    pub(crate) solved_without_assumptions: bool,
}

impl SolvabilityStatsDto {
    #[must_use]
    pub(crate) fn from_stats(stats: &BacktrackSolverStats) -> Self {
        Self {
            assumptions_len: stats.assumptions().len(),
            backtrack_count: stats.backtrack_count(),
            solved_without_assumptions: stats.solved_without_assumptions(),
        }
    }
}

/// Compact candidate grid DTO.
///
/// Stores a 9-bit mask per cell, ordered by `Position::ALL`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SolvabilityGridDto {
    pub(crate) candidates: Vec<u16>,
}

impl SolvabilityGridDto {
    /// Builds a DTO from a [`CandidateGrid`].
    #[must_use]
    pub(crate) fn from_candidate_grid(grid: &CandidateGrid) -> Self {
        let mut candidates = Vec::with_capacity(81);
        for pos in Position::ALL {
            let set = grid.candidates_at(pos);
            candidates.push(set.bits());
        }
        Self { candidates }
    }

    /// Converts this DTO into a [`CandidateGrid`].
    ///
    /// # Errors
    ///
    /// Returns [`SolvabilityDtoError::InvalidCandidateLength`] if the candidate
    /// length does not match the expected 81 cells.
    ///
    /// Returns [`SolvabilityDtoError::InvalidCandidateBits`] if any cell contains
    /// invalid bits outside the 9-bit candidate range.
    pub(crate) fn to_candidate_grid(&self) -> Result<CandidateGrid, SolvabilityDtoError> {
        if self.candidates.len() != 81 {
            return Err(SolvabilityDtoError::InvalidCandidateLength {
                len: self.candidates.len(),
            });
        }

        let mut grid = CandidateGrid::new();
        for (idx, pos) in Position::ALL.into_iter().enumerate() {
            let bits = self.candidates[idx];
            let candidates = DigitSet::try_from_bits(bits)
                .ok_or(SolvabilityDtoError::InvalidCandidateBits { pos, bits })?;
            for digit in Digit::ALL {
                if !candidates.contains(digit) {
                    grid.remove_candidate(pos, digit);
                }
            }
        }
        Ok(grid)
    }
}

/// Errors that can occur when converting solvability DTOs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display, derive_more::Error)]
pub(crate) enum SolvabilityDtoError {
    #[display("invalid candidate length: {len}")]
    InvalidCandidateLength { len: usize },
    #[display("invalid candidate bits for {pos:?}: {bits:#05x}")]
    InvalidCandidateBits { pos: Position, bits: u16 },
}
