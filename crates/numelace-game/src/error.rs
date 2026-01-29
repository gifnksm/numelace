use derive_more::{Display, Error};

/// Errors that can occur during game operations.
#[derive(Debug, Display, Error)]
pub enum GameError {
    /// Attempted to modify a given (initial) cell.
    ///
    /// Given cells are part of the initial puzzle and cannot be edited by the player.
    #[display("cannot modify a given cell")]
    CannotModifyGivenCell,
    /// Attempted to add notes to a filled cell.
    ///
    /// Notes can only be added to notes or empty cells.
    #[display("cannot add notes to a filled cell")]
    CannotAddNoteToFilledCell,
    /// Invalid notes data provided.
    ///
    /// The notes data must be a valid bitmask representing digits 1-9.
    #[display("invalid notes data: {_0:#x}")]
    InvalidNotes(#[error(not(source))] u16),
    /// Attempted to apply a digit that conflicts with existing digits.
    ///
    /// This occurs when the digit violates Sudoku rules in strict mode.
    #[display("given digit causes a conflict with existing digits")]
    ConflictingDigit,
}
