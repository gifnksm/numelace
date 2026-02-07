pub(crate) mod generate_puzzle;
pub(crate) mod solvability;

pub(crate) use generate_puzzle::*;
use numelace_core::{CandidateGrid, Digit, DigitSet, Position};
use numelace_game::Game;
use serde::{Deserialize, Serialize};
pub(crate) use solvability::*;

/// Compact candidate grid DTO.
///
/// Stores a 9-bit mask per cell, ordered by `Position::ALL`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CandidateGridDto {
    pub(crate) candidates: Vec<u16>,
}

impl From<CandidateGrid> for CandidateGridDto {
    fn from(grid: CandidateGrid) -> Self {
        let mut candidates = Vec::with_capacity(81);
        for pos in Position::ALL {
            let set = grid.candidates_at(pos);
            candidates.push(set.bits());
        }
        Self { candidates }
    }
}

/// Errors that can occur when converting solvability DTOs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display, derive_more::Error)]
pub(crate) enum CandidateGridDtoError {
    #[display("invalid candidate length: {len}")]
    InvalidCandidateLength { len: usize },
    #[display("invalid candidate bits for {pos:?}: {bits:#05x}")]
    InvalidCandidateBits { pos: Position, bits: u16 },
}

/// Converts a [`CandidateGridDto`] into a [`CandidateGrid`].
///
/// # Errors
///
/// Returns [`CandidateGridDtoError::InvalidCandidateLength`] if the candidate
/// length does not match the expected 81 cells.
///
/// Returns [`CandidateGridDtoError::InvalidCandidateBits`] if any cell contains
/// invalid bits outside the 9-bit candidate range.
impl TryFrom<CandidateGridDto> for CandidateGrid {
    type Error = CandidateGridDtoError;

    fn try_from(dto: CandidateGridDto) -> Result<Self, Self::Error> {
        if dto.candidates.len() != 81 {
            return Err(CandidateGridDtoError::InvalidCandidateLength {
                len: dto.candidates.len(),
            });
        }

        let mut grid = CandidateGrid::new();
        for (idx, pos) in Position::ALL.into_iter().enumerate() {
            let bits = dto.candidates[idx];
            let candidates = DigitSet::try_from_bits(bits)
                .ok_or(CandidateGridDtoError::InvalidCandidateBits { pos, bits })?;
            for digit in Digit::ALL {
                if !candidates.contains(digit) {
                    grid.remove_candidate(pos, digit);
                }
            }
        }
        Ok(grid)
    }
}

/// DTO containing candidate grids for solvability checks.
///
/// The task first checks the grid with user notes. If that is inconsistent or
/// has no solution, it falls back to the grid that keeps only decided cells.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CandidateGridPairDto {
    pub(crate) with_user_notes: CandidateGridDto,
    pub(crate) without_user_notes: CandidateGridDto,
}

impl From<&Game> for CandidateGridPairDto {
    fn from(game: &Game) -> Self {
        Self {
            with_user_notes: game.to_candidate_grid_with_notes().into(),
            without_user_notes: game.to_candidate_grid().into(),
        }
    }
}

impl From<Game> for CandidateGridPairDto {
    fn from(game: Game) -> Self {
        Self::from(&game)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CandidateGridPairsDto {
    pub(crate) grids: Vec<CandidateGridPairDto>,
}

impl From<Vec<Game>> for CandidateGridPairsDto {
    fn from(games: Vec<Game>) -> Self {
        let grids = games.into_iter().map(CandidateGridPairDto::from).collect();
        Self { grids }
    }
}
