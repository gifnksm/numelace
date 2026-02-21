//! Solvability task logic and DTOs.
//!
//! This module provides compact, serializable representations of candidate grids
//! so they can be sent across worker boundaries. It also includes the solvability
//! logic used by background tasks.

use numelace_core::CandidateGrid;
use numelace_solver::{BacktrackSolverStats, TechniqueGrid, technique};
use serde::{Deserialize, Serialize};

use crate::worker::tasks::{CandidateGridDtoError, CandidateGridPairDto, CandidateGridPairsDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TechniqueCountDto {
    pub(crate) name: String,
    pub(crate) count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SolvabilityStatsDto {
    pub(crate) assumptions_len: usize,
    pub(crate) backtrack_count: usize,
    pub(crate) solved_without_assumptions: bool,
    pub(crate) total_steps: usize,
    pub(crate) technique_counts: Vec<TechniqueCountDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum SolvabilityResultDto {
    Inconsistent,
    NoSolution,
    Solvable {
        with_user_notes: bool,
        stats: SolvabilityStatsDto,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SolvabilityUndoScanResultDto {
    pub(crate) index: Option<usize>,
    pub(crate) state: SolvabilityResultDto,
}

impl From<BacktrackSolverStats> for SolvabilityStatsDto {
    fn from(stats: BacktrackSolverStats) -> Self {
        let technique_counts = technique::all_techniques()
            .iter()
            .zip(stats.technique().applications().iter())
            .map(|(tech, count)| TechniqueCountDto {
                name: tech.name().to_string(),
                count: *count,
            })
            .collect();

        Self {
            assumptions_len: stats.assumptions().len(),
            backtrack_count: stats.backtrack_count(),
            solved_without_assumptions: stats.solved_without_assumptions(),
            total_steps: stats.technique().total_steps(),
            technique_counts,
        }
    }
}

/// Scan undo history grids to find the first solvable state.
pub(crate) fn handle_solvability_undo_scan(
    request: CandidateGridPairsDto,
) -> Result<SolvabilityUndoScanResultDto, CandidateGridDtoError> {
    for (index, grids) in request.grids.into_iter().enumerate() {
        let with_user_notes = TechniqueGrid::from(CandidateGrid::try_from(grids.with_user_notes)?);
        let without_user_notes =
            TechniqueGrid::from(CandidateGrid::try_from(grids.without_user_notes)?);

        let with_state = check_grid_solvability(with_user_notes, true);
        if matches!(with_state, SolvabilityResultDto::Solvable { .. }) {
            return Ok(SolvabilityUndoScanResultDto {
                index: Some(index),
                state: with_state,
            });
        }

        let without_state = check_grid_solvability(without_user_notes, false);
        if matches!(without_state, SolvabilityResultDto::Solvable { .. }) {
            return Ok(SolvabilityUndoScanResultDto {
                index: Some(index),
                state: without_state,
            });
        }
    }

    Ok(SolvabilityUndoScanResultDto {
        index: None,
        state: SolvabilityResultDto::NoSolution,
    })
}

/// Runs solvability logic with fallback between user notes and raw candidates.
///
/// If the `with_user_notes` grid is inconsistent or unsolvable, the task retries
/// with a grid that keeps only decided cells.
pub(crate) fn handle_solvability_request(
    request: CandidateGridPairDto,
) -> Result<SolvabilityResultDto, CandidateGridDtoError> {
    let with_user_notes = TechniqueGrid::from(CandidateGrid::try_from(request.with_user_notes)?);
    let without_user_notes =
        TechniqueGrid::from(CandidateGrid::try_from(request.without_user_notes)?);

    let first_result = check_grid_solvability(with_user_notes, true);
    let result = if matches!(
        first_result,
        SolvabilityResultDto::Inconsistent | SolvabilityResultDto::NoSolution
    ) {
        check_grid_solvability(without_user_notes, false)
    } else {
        first_result
    };

    Ok(result)
}

fn check_grid_solvability(grid: TechniqueGrid, with_user_notes: bool) -> SolvabilityResultDto {
    if grid.check_consistency().is_err() {
        return SolvabilityResultDto::Inconsistent;
    }

    let solver = numelace_solver::BacktrackSolver::with_all_techniques();
    match solver.solve(grid).map(|mut sol| sol.next()) {
        Ok(Some((_grid, stats))) => SolvabilityResultDto::Solvable {
            with_user_notes,
            stats: stats.into(),
        },
        Ok(None) | Err(_) => SolvabilityResultDto::NoSolution,
    }
}
