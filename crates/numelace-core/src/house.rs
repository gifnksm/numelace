use std::iter::FusedIterator;

use crate::{
    Digit, DigitPositions, Position,
    index::{DigitSemantics, Index9, Index9Semantics as _},
};

/// A Sudoku house (row, column, or 3×3 box).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum House {
    /// A row identified by its y coordinate (0-8).
    Row {
        /// Row index (0-8).
        y: u8,
    },
    /// A column identified by its x coordinate (0-8).
    Column {
        /// Column index (0-8).
        x: u8,
    },
    /// A 3×3 box identified by its index (0-8, left to right, top to bottom).
    Box {
        /// Box index (0-8).
        index: u8,
    },
}

impl House {
    /// Array containing all rows (0-8).
    pub const ROWS: [Self; 9] = [
        Self::Row { y: 0 },
        Self::Row { y: 1 },
        Self::Row { y: 2 },
        Self::Row { y: 3 },
        Self::Row { y: 4 },
        Self::Row { y: 5 },
        Self::Row { y: 6 },
        Self::Row { y: 7 },
        Self::Row { y: 8 },
    ];

    /// Array containing all columns (0-8).
    pub const COLUMNS: [Self; 9] = [
        Self::Column { x: 0 },
        Self::Column { x: 1 },
        Self::Column { x: 2 },
        Self::Column { x: 3 },
        Self::Column { x: 4 },
        Self::Column { x: 5 },
        Self::Column { x: 6 },
        Self::Column { x: 7 },
        Self::Column { x: 8 },
    ];

    /// Array containing all boxes (0-8).
    pub const BOXES: [Self; 9] = [
        Self::Box { index: 0 },
        Self::Box { index: 1 },
        Self::Box { index: 2 },
        Self::Box { index: 3 },
        Self::Box { index: 4 },
        Self::Box { index: 5 },
        Self::Box { index: 6 },
        Self::Box { index: 7 },
        Self::Box { index: 8 },
    ];

    /// Array containing all houses in row, column, box order.
    pub const ALL: [Self; 27] = {
        let mut all = [Self::Row { y: 0 }; 27];
        let mut i = 0;
        #[expect(clippy::cast_possible_truncation)]
        while i < 9 {
            all[i] = Self::Row { y: i as u8 };
            all[i + 9] = Self::Column { x: i as u8 };
            all[i + 18] = Self::Box { index: i as u8 };
            i += 1;
        }
        all
    };

    /// Converts a cell index within the house (0-8) into an absolute [`Position`].
    ///
    /// # Panics
    ///
    /// Panics if `i` is not in the range 0-8.
    #[must_use]
    #[inline]
    pub fn position_from_cell_index(self, i: u8) -> Position {
        assert!(i < 9);
        match self {
            House::Row { y } => Position::new(i, y),
            House::Column { x } => Position::new(x, i),
            House::Box { index } => Position::from_box(index, i),
        }
    }

    /// Returns all positions contained in this house.
    #[must_use]
    pub fn positions(self) -> DigitPositions {
        match self {
            House::Row { y } => DigitPositions::ROW_POSITIONS[y],
            House::Column { x } => DigitPositions::COLUMN_POSITIONS[x],
            House::Box { index } => DigitPositions::BOX_POSITIONS[index],
        }
    }
}

/// Returns an iterator over all `(Digit, House)` pairs.
///
/// The iteration order is digit-major: for each digit in [`Digit::ALL`], it yields every
/// house in [`House::ALL`] (rows, columns, boxes).
#[must_use]
#[inline]
pub fn all_digit_houses() -> AllDigitHouses {
    AllDigitHouses {
        front: (0, 0, 0),
        back: (9, 0, 0),
    }
}

/// Iterator over all `(Digit, House)` pairs.
#[derive(Debug, Clone)]
pub struct AllDigitHouses {
    front: (u8, u8, u8),
    back: (u8, u8, u8),
}

impl AllDigitHouses {
    #[inline]
    fn to_linear(digit: u8, house: u8, index: u8) -> u16 {
        u16::from(digit) * 27 + u16::from(house) * 9 + u16::from(index)
    }

    #[inline]
    fn item_at(digit: u8, house: u8, index: u8) -> (Digit, House) {
        debug_assert!(digit < 9);
        debug_assert!(house < 3);
        debug_assert!(index < 9);
        let digit = DigitSemantics::from_index(Index9::new(digit));
        let house = match house {
            0 => House::Row { y: index },
            1 => House::Column { x: index },
            2 => House::Box { index },
            _ => unreachable!(),
        };
        (digit, house)
    }
}

impl Iterator for AllDigitHouses {
    type Item = (Digit, House);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }
        let (digit, house, index) = &mut self.front;
        let item = Self::item_at(*digit, *house, *index);
        *index += 1;
        if *index == 9 {
            *index = 0;
            *house += 1;
        }
        if *house == 3 {
            *house = 0;
            *digit += 1;
        }
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let front = Self::to_linear(self.front.0, self.front.1, self.front.2);
        let back = Self::to_linear(self.back.0, self.back.1, self.back.2);
        let remaining = back.saturating_sub(front);
        (usize::from(remaining), Some(usize::from(remaining)))
    }
}

impl DoubleEndedIterator for AllDigitHouses {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }
        let (digit, house, index) = &mut self.back;
        if *index > 0 {
            *index -= 1;
        } else {
            *index = 8;
            if *house > 0 {
                *house -= 1;
            } else {
                *house = 2;
                *digit -= 1;
            }
        }
        Some(Self::item_at(*digit, *house, *index))
    }
}

impl FusedIterator for AllDigitHouses {}
impl ExactSizeIterator for AllDigitHouses {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_digit_houses_iterator_order() {
        let mut iter = all_digit_houses();
        assert_eq!(iter.next(), Some((Digit::D1, House::Row { y: 0 })));
        assert_eq!(iter.next_back(), Some((Digit::D9, House::Box { index: 8 })));
        assert_eq!(iter.len(), 9 * 27 - 2);
    }
}
