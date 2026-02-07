use numelace_core::DigitGrid;
use numelace_generator::GeneratedPuzzle;
use serde::{Deserialize, Serialize};

/// DTO for communicating newly generated Sudoku puzzles over worker boundaries.
///
/// Uses compact 81-char string formats ('.' for empty, '1'..'9' for digits).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NewGameDto {
    pub(crate) seed: String,
    pub(crate) problem: String,
    pub(crate) solution: String,
}

impl From<GeneratedPuzzle> for NewGameDto {
    fn from(puzzle: GeneratedPuzzle) -> Self {
        Self {
            seed: puzzle.seed.to_string(),
            problem: puzzle.problem.to_string(),
            solution: puzzle.solution.to_string(),
        }
    }
}

impl TryFrom<NewGameDto> for GeneratedPuzzle {
    type Error = String;

    fn try_from(value: NewGameDto) -> Result<Self, Self::Error> {
        let seed = value.seed.parse()?;
        let problem = value
            .problem
            .parse::<DigitGrid>()
            .map_err(|e| e.to_string())?;
        let solution = value
            .solution
            .parse::<DigitGrid>()
            .map_err(|e| e.to_string())?;
        Ok(GeneratedPuzzle {
            seed,
            problem,
            solution,
        })
    }
}
