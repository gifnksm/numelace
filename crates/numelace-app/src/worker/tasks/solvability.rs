//! Solvability task logic and DTOs.
//!
//! This module provides compact, serializable representations of candidate grids
//! so they can be sent across worker boundaries. It also includes the solvability
//! logic used by background tasks.

use numelace_core::CandidateGrid;
use numelace_solver::BacktrackSolverStats;
use serde::{Deserialize, Serialize};

use crate::{
    state::{SolvabilityState, SolvabilityStats},
    worker::tasks::{CandidateGridDtoError, CandidateGridPairDto, CandidateGridPairsDto},
};

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

impl From<SolvabilityStateDto> for SolvabilityState {
    fn from(value: SolvabilityStateDto) -> Self {
        match value {
            SolvabilityStateDto::Inconsistent => Self::Inconsistent,
            SolvabilityStateDto::NoSolution => Self::NoSolution,
            SolvabilityStateDto::Solvable {
                with_user_notes,
                stats,
            } => Self::Solvable {
                with_user_notes,
                stats: stats.into(),
            },
        }
    }
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

impl From<SolvabilityStatsDto> for SolvabilityStats {
    fn from(stats: SolvabilityStatsDto) -> Self {
        SolvabilityStats {
            assumptions_len: stats.assumptions_len,
            backtrack_count: stats.backtrack_count,
            solved_without_assumptions: stats.solved_without_assumptions,
        }
    }
}

/// Scan undo history grids to find the first solvable state.
pub(crate) fn handle_solvability_undo_scan(
    request: CandidateGridPairsDto,
) -> Result<SolvabilityUndoScanResultDto, CandidateGridDtoError> {
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
    request: CandidateGridPairDto,
) -> Result<SolvabilityStateDto, CandidateGridDtoError> {
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
