use numelace_core::{Digit, DigitSet};

use crate::{error::GameError, input::InputCapability};

/// The state of a cell in the game.
///
/// This enum distinguishes between four types of cells:
/// - [`Given`]: Initial puzzle cells (cannot be modified)
/// - [`Filled`]: Player-filled cells (can be modified or cleared)
/// - [`Notes`]: Candidate notes (player memo)
/// - [`Empty`]: Cells that have not been filled yet
///
/// [`Given`]: CellState::Given
/// [`Filled`]: CellState::Filled
/// [`Notes`]: CellState::Notes
/// [`Empty`]: CellState::Empty
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::IsVariant)]
pub enum CellState {
    /// A cell from the initial puzzle (cannot be modified by the player).
    Given(Digit),
    /// A cell filled by the player (can be modified or cleared).
    Filled(Digit),
    /// Candidate notes for this cell.
    Notes(DigitSet),
    /// An empty cell (not yet filled).
    Empty,
}

impl CellState {
    /// Returns whether setting a digit is allowed for this cell.
    ///
    /// This only checks cell-local constraints (e.g., given cells are immutable).
    /// Board-level conflicts are handled by [`crate::Game`].
    #[must_use]
    pub(crate) fn set_digit_capability(self, _digit: Digit) -> InputCapability {
        match self {
            CellState::Given(_) => InputCapability::BlockedByGivenCell,
            CellState::Filled(_) | CellState::Notes(_) | CellState::Empty => {
                InputCapability::Allowed
            }
        }
    }

    /// Returns whether toggling a note is allowed for this cell.
    ///
    /// This only checks cell-local constraints (e.g., filled cells cannot hold notes).
    /// Board-level conflicts are handled by [`crate::Game`].
    #[must_use]
    pub(crate) fn toggle_note_capability(self, _digit: Digit) -> InputCapability {
        match self {
            CellState::Given(_) => InputCapability::BlockedByGivenCell,
            CellState::Filled(_) => InputCapability::BlockedByFilledCell,
            CellState::Notes(_) | CellState::Empty => InputCapability::Allowed,
        }
    }

    /// Returns whether setting the given digit is a no-op for this cell.
    #[must_use]
    pub(crate) fn is_same_filled_digit(self, digit: Digit) -> bool {
        matches!(self, CellState::Filled(existing) if existing == digit)
    }

    /// Returns whether toggling this digit would remove an existing note.
    #[must_use]
    pub(crate) fn is_note_removal(self, digit: Digit) -> bool {
        matches!(self, CellState::Notes(notes) if notes.contains(digit))
    }

    /// Sets this cell to a filled digit, clearing notes if needed.
    ///
    /// # Errors
    ///
    /// Returns [`GameError::CannotModifyGivenCell`] if this is a given cell.
    pub fn set_filled(&mut self, digit: Digit) -> Result<(), GameError> {
        match self {
            CellState::Given(_) => Err(GameError::CannotModifyGivenCell),
            CellState::Filled(d) => {
                *d = digit;
                Ok(())
            }
            CellState::Notes(_) | CellState::Empty => {
                *self = CellState::Filled(digit);
                Ok(())
            }
        }
    }

    /// Drops a digit from this cell's notes, converting empty notes to [`CellState::Empty`].
    pub fn drop_note_digit(&mut self, digit: Digit) {
        if let CellState::Notes(notes) = self {
            notes.remove(digit);
            if notes.is_empty() {
                *self = CellState::Empty;
            }
        }
    }

    /// Toggles a note in this cell.
    ///
    /// # Errors
    ///
    /// Returns [`GameError::CannotModifyGivenCell`] if this is a given cell.
    /// Returns [`GameError::CannotAddNoteToFilledCell`] if this is a filled cell.
    pub fn toggle_note(&mut self, digit: Digit) -> Result<(), GameError> {
        match self {
            CellState::Given(_) => Err(GameError::CannotModifyGivenCell),
            CellState::Filled(_) => Err(GameError::CannotAddNoteToFilledCell),
            CellState::Notes(digits) => {
                digits.toggle(digit);
                if digits.is_empty() {
                    *self = CellState::Empty;
                }
                Ok(())
            }
            CellState::Empty => {
                let mut digits = DigitSet::new();
                digits.insert(digit);
                *self = CellState::Notes(digits);
                Ok(())
            }
        }
    }

    /// Clears this cell if it contains player input or notes.
    ///
    /// # Errors
    ///
    /// Returns [`GameError::CannotModifyGivenCell`] if this is a given cell.
    pub fn clear(&mut self) -> Result<(), GameError> {
        match self {
            CellState::Given(_) => Err(GameError::CannotModifyGivenCell),
            CellState::Filled(_) | CellState::Notes(_) => {
                *self = CellState::Empty;
                Ok(())
            }
            CellState::Empty => Ok(()),
        }
    }

    /// Returns the digit if this is a given cell, otherwise `None`.
    #[must_use]
    pub fn as_given(&self) -> Option<Digit> {
        match self {
            CellState::Given(digit) => Some(*digit),
            _ => None,
        }
    }

    /// Returns the digit if this is a filled cell, otherwise `None`.
    #[must_use]
    pub fn as_filled(&self) -> Option<Digit> {
        match self {
            CellState::Filled(digit) => Some(*digit),
            _ => None,
        }
    }

    /// Returns the notes if this is a notes cell, otherwise `None`.
    #[must_use]
    pub fn as_notes(&self) -> Option<DigitSet> {
        match self {
            CellState::Notes(digits) => Some(*digits),
            _ => None,
        }
    }

    /// Returns the digit if this cell contains one (given or filled), otherwise `None`.
    #[must_use]
    pub fn as_digit(&self) -> Option<Digit> {
        match self {
            CellState::Given(digit) | CellState::Filled(digit) => Some(*digit),
            CellState::Notes(_) | CellState::Empty => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::{Digit, DigitSet};

    use super::*;

    #[test]
    fn test_cell_state_helpers() {
        // as_given
        assert_eq!(CellState::Given(Digit::D5).as_given(), Some(Digit::D5));
        assert_eq!(CellState::Filled(Digit::D5).as_given(), None);
        assert_eq!(CellState::Empty.as_given(), None);

        // as_filled
        assert_eq!(CellState::Filled(Digit::D5).as_filled(), Some(Digit::D5));
        assert_eq!(CellState::Given(Digit::D5).as_filled(), None);
        assert_eq!(CellState::Empty.as_filled(), None);

        // as_notes
        let mut notes = DigitSet::new();
        notes.insert(Digit::D3);
        assert_eq!(CellState::Notes(notes).as_notes(), Some(notes));
        assert_eq!(CellState::Empty.as_notes(), None);

        // as_digit (unified access)
        assert_eq!(CellState::Given(Digit::D5).as_digit(), Some(Digit::D5));
        assert_eq!(CellState::Filled(Digit::D7).as_digit(), Some(Digit::D7));
        assert_eq!(CellState::Notes(notes).as_digit(), None);
        assert_eq!(CellState::Empty.as_digit(), None);
    }

    #[test]
    fn test_cell_state_set_filled_transitions() {
        let mut cell = CellState::Empty;
        cell.set_filled(Digit::D4).unwrap();
        assert_eq!(cell, CellState::Filled(Digit::D4));

        let mut cell = CellState::Notes({
            let mut notes = DigitSet::new();
            notes.insert(Digit::D2);
            notes
        });
        cell.set_filled(Digit::D8).unwrap();
        assert_eq!(cell, CellState::Filled(Digit::D8));
    }

    #[test]
    fn test_cell_state_drop_note_digit_clears_empty() {
        let mut notes = DigitSet::new();
        notes.insert(Digit::D6);
        let mut cell = CellState::Notes(notes);
        cell.drop_note_digit(Digit::D6);
        assert_eq!(cell, CellState::Empty);
    }

    #[test]
    fn test_cell_state_toggle_note_transitions() {
        let mut cell = CellState::Empty;
        cell.toggle_note(Digit::D3).unwrap();
        assert!(matches!(
            cell,
            CellState::Notes(notes) if notes.contains(Digit::D3)
        ));

        cell.toggle_note(Digit::D3).unwrap();
        assert_eq!(cell, CellState::Empty);
    }

    #[test]
    fn test_cell_state_clear_transitions() {
        let mut cell = CellState::Filled(Digit::D1);
        cell.clear().unwrap();
        assert_eq!(cell, CellState::Empty);

        let mut notes = DigitSet::new();
        notes.insert(Digit::D9);
        let mut cell = CellState::Notes(notes);
        cell.clear().unwrap();
        assert_eq!(cell, CellState::Empty);
    }

    #[test]
    fn test_cell_state_is_variant() {
        // derive_more::IsVariant generates these methods
        assert!(CellState::Given(Digit::D5).is_given());
        assert!(!CellState::Given(Digit::D5).is_filled());
        assert!(!CellState::Given(Digit::D5).is_empty());

        assert!(CellState::Filled(Digit::D5).is_filled());
        assert!(!CellState::Filled(Digit::D5).is_given());
        assert!(!CellState::Filled(Digit::D5).is_empty());

        assert!(CellState::Empty.is_empty());
        assert!(!CellState::Empty.is_given());
        assert!(!CellState::Empty.is_filled());
    }
}
