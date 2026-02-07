//! Solvability task logic and DTOs.
//!
//! This module provides compact, serializable representations of candidate grids
//! so they can be sent across worker boundaries. It also includes the solvability
//! logic used by background tasks.

use numelace_core::{CandidateGrid, Digit, DigitSet, Position};
use numelace_solver::BacktrackSolverStats;
use serde::{Deserialize, Serialize};

/// DTO containing candidate grids for solvability checks.
///
/// The task first checks the grid with user notes. If that is inconsistent or
/// has no solution, it falls back to the grid that keeps only decided cells.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SolvabilityRequestDto {
    pub(crate) with_user_notes: SolvabilityGridDto,
    pub(crate) without_user_notes: SolvabilityGridDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SolvabilityUndoGridsDto {
    pub(crate) grids: Vec<SolvabilityRequestDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SolvabilityUndoScanResultDto {
    pub(crate) index: Option<usize>,
    pub(crate) state: SolvabilityStateDto,
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

impl From<BacktrackSolverStats> for SolvabilityStatsDto {
    fn from(stats: BacktrackSolverStats) -> Self {
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

impl From<CandidateGrid> for SolvabilityGridDto {
    fn from(grid: CandidateGrid) -> Self {
        let mut candidates = Vec::with_capacity(81);
        for pos in Position::ALL {
            let set = grid.candidates_at(pos);
            candidates.push(set.bits());
        }
        Self { candidates }
    }
}

/// Converts a [`SolvabilityGridDto`] into a [`CandidateGrid`].
///
/// # Errors
///
/// Returns [`SolvabilityDtoError::InvalidCandidateLength`] if the candidate
/// length does not match the expected 81 cells.
///
/// Returns [`SolvabilityDtoError::InvalidCandidateBits`] if any cell contains
/// invalid bits outside the 9-bit candidate range.
impl TryFrom<SolvabilityGridDto> for CandidateGrid {
    type Error = SolvabilityDtoError;

    fn try_from(dto: SolvabilityGridDto) -> Result<Self, Self::Error> {
        if dto.candidates.len() != 81 {
            return Err(SolvabilityDtoError::InvalidCandidateLength {
                len: dto.candidates.len(),
            });
        }

        let mut grid = CandidateGrid::new();
        for (idx, pos) in Position::ALL.into_iter().enumerate() {
            let bits = dto.candidates[idx];
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

/// Scan undo history grids to find the first solvable state.
pub(crate) fn handle_solvability_undo_scan(
    request: SolvabilityUndoGridsDto,
) -> Result<SolvabilityUndoScanResultDto, SolvabilityDtoError> {
    for (index, grids) in request.grids.into_iter().enumerate() {
        let with_user_notes: CandidateGrid = grids.with_user_notes.try_into()?;
        let without_user_notes: CandidateGrid = grids.without_user_notes.try_into()?;

        let with_state = check_grid_solvability(with_user_notes, true);
        if matches!(with_state, SolvabilityStateDto::Solvable { .. }) {
            return Ok(SolvabilityUndoScanResultDto {
                index: Some(index),
                state: with_state,
            });
        }

        let without_state = check_grid_solvability(without_user_notes, false);
        if matches!(without_state, SolvabilityStateDto::Solvable { .. }) {
            return Ok(SolvabilityUndoScanResultDto {
                index: Some(index),
                state: without_state,
            });
        }
    }

    Ok(SolvabilityUndoScanResultDto {
        index: None,
        state: SolvabilityStateDto::NoSolution,
    })
}

/// Runs solvability logic with fallback between user notes and raw candidates.
///
/// If the `with_user_notes` grid is inconsistent or unsolvable, the task retries
/// with a grid that keeps only decided cells.
pub(crate) fn handle_solvability_request(
    request: SolvabilityRequestDto,
) -> Result<SolvabilityStateDto, SolvabilityDtoError> {
    let with_user_notes: CandidateGrid = request.with_user_notes.try_into()?;
    let without_user_notes: CandidateGrid = request.without_user_notes.try_into()?;

    let first_result = check_grid_solvability(with_user_notes, true);
    let result = if matches!(
        first_result,
        SolvabilityStateDto::Inconsistent | SolvabilityStateDto::NoSolution
    ) {
        check_grid_solvability(without_user_notes, false)
    } else {
        first_result
    };

    Ok(result)
}

fn check_grid_solvability(grid: CandidateGrid, with_user_notes: bool) -> SolvabilityStateDto {
    if grid.check_consistency().is_err() {
        return SolvabilityStateDto::Inconsistent;
    }

    let solver = numelace_solver::BacktrackSolver::with_all_techniques();
    match solver.solve(grid).map(|mut sol| sol.next()) {
        Ok(Some((_grid, stats))) => SolvabilityStateDto::Solvable {
            with_user_notes,
            stats: stats.into(),
        },
        Ok(None) | Err(_) => SolvabilityStateDto::NoSolution,
    }
}
