use serde::{Deserialize, Serialize};

/// DTO for communicating newly generated Sudoku puzzles over worker boundaries.
///
/// Uses compact 81-char string formats ('.' for empty, '1'..'9' for digits).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NewGameDto {
    pub(crate) problem: String,
    pub(crate) solution: String,
}
