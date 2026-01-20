//! Sudoku digit representation.
//!
//! This module provides the [`Digit`] enum, which represents the digits 1-9
//! used in sudoku puzzles with type safety and compile-time guarantees.

use std::fmt::{self, Display};

/// A sudoku digit in the range 1-9.
///
/// This enum provides type-safe representation of sudoku digits, preventing
/// invalid values at compile time. Each variant corresponds to exactly one
/// digit value.
///
/// # Examples
///
/// ```
/// use sudoku_core::Digit;
///
/// let digit = Digit::D5;
/// assert_eq!(digit.value(), 5);
///
/// // Create from a u8 value
/// let digit = Digit::from_value(7);
/// assert_eq!(digit, Digit::D7);
///
/// // Iterate over all digits
/// for digit in Digit::ALL {
///     println!("{}", digit);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Digit {
    D1 = 1,
    D2 = 2,
    D3 = 3,
    D4 = 4,
    D5 = 5,
    D6 = 6,
    D7 = 7,
    D8 = 8,
    D9 = 9,
}

impl Digit {
    /// Array containing all digits from 1 to 9.
    ///
    /// Useful for iterating over all possible sudoku digits.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::Digit;
    ///
    /// assert_eq!(Digit::ALL.len(), 9);
    /// assert_eq!(Digit::ALL[0], Digit::D1);
    /// assert_eq!(Digit::ALL[8], Digit::D9);
    ///
    /// // Iterate over all digits
    /// for digit in Digit::ALL {
    ///     assert!((1..=9).contains(&digit.value()));
    /// }
    /// ```
    pub const ALL: [Self; 9] = [
        Self::D1,
        Self::D2,
        Self::D3,
        Self::D4,
        Self::D5,
        Self::D6,
        Self::D7,
        Self::D8,
        Self::D9,
    ];

    /// Creates a digit from a u8 value in the range 1-9.
    ///
    /// # Panics
    ///
    /// Panics if `value` is not in the range 1-9.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::Digit;
    ///
    /// let digit = Digit::from_value(5);
    /// assert_eq!(digit, Digit::D5);
    ///
    /// let digit = Digit::from_value(1);
    /// assert_eq!(digit, Digit::D1);
    /// ```
    ///
    /// ```should_panic
    /// use sudoku_core::Digit;
    ///
    /// // This will panic
    /// let _ = Digit::from_value(0);
    /// ```
    #[must_use]
    pub fn from_value(value: u8) -> Self {
        match value {
            1 => Self::D1,
            2 => Self::D2,
            3 => Self::D3,
            4 => Self::D4,
            5 => Self::D5,
            6 => Self::D6,
            7 => Self::D7,
            8 => Self::D8,
            9 => Self::D9,
            _ => panic!("Invalid digit value: {value}"),
        }
    }

    /// Returns the numeric value of this digit (1-9).
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::Digit;
    ///
    /// assert_eq!(Digit::D1.value(), 1);
    /// assert_eq!(Digit::D5.value(), 5);
    /// assert_eq!(Digit::D9.value(), 9);
    /// ```
    #[must_use]
    pub const fn value(&self) -> u8 {
        *self as u8
    }
}

impl Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value(), f)
    }
}

impl From<Digit> for u8 {
    fn from(digit: Digit) -> u8 {
        digit.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_value_valid() {
        assert_eq!(Digit::from_value(1), Digit::D1);
        assert_eq!(Digit::from_value(2), Digit::D2);
        assert_eq!(Digit::from_value(3), Digit::D3);
        assert_eq!(Digit::from_value(4), Digit::D4);
        assert_eq!(Digit::from_value(5), Digit::D5);
        assert_eq!(Digit::from_value(6), Digit::D6);
        assert_eq!(Digit::from_value(7), Digit::D7);
        assert_eq!(Digit::from_value(8), Digit::D8);
        assert_eq!(Digit::from_value(9), Digit::D9);
    }

    #[test]
    #[should_panic(expected = "Invalid digit value: 0")]
    fn test_from_value_zero_panics() {
        let _ = Digit::from_value(0);
    }

    #[test]
    #[should_panic(expected = "Invalid digit value: 10")]
    fn test_from_value_ten_panics() {
        let _ = Digit::from_value(10);
    }

    #[test]
    fn test_value() {
        assert_eq!(Digit::D1.value(), 1);
        assert_eq!(Digit::D2.value(), 2);
        assert_eq!(Digit::D3.value(), 3);
        assert_eq!(Digit::D4.value(), 4);
        assert_eq!(Digit::D5.value(), 5);
        assert_eq!(Digit::D6.value(), 6);
        assert_eq!(Digit::D7.value(), 7);
        assert_eq!(Digit::D8.value(), 8);
        assert_eq!(Digit::D9.value(), 9);
    }

    #[test]
    fn test_all_constant() {
        assert_eq!(Digit::ALL.len(), 9);
        assert_eq!(Digit::ALL[0], Digit::D1);
        assert_eq!(Digit::ALL[1], Digit::D2);
        assert_eq!(Digit::ALL[2], Digit::D3);
        assert_eq!(Digit::ALL[3], Digit::D4);
        assert_eq!(Digit::ALL[4], Digit::D5);
        assert_eq!(Digit::ALL[5], Digit::D6);
        assert_eq!(Digit::ALL[6], Digit::D7);
        assert_eq!(Digit::ALL[7], Digit::D8);
        assert_eq!(Digit::ALL[8], Digit::D9);
    }

    #[test]
    fn test_all_iteration() {
        let mut count = 0;
        for (i, digit) in Digit::ALL.iter().enumerate() {
            assert_eq!(digit.value() as usize, i + 1);
            count += 1;
        }
        assert_eq!(count, 9);
    }

    #[test]
    fn test_round_trip() {
        for digit in Digit::ALL {
            let value = digit.value();
            let reconstructed = Digit::from_value(value);
            assert_eq!(digit, reconstructed);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Digit::D1), "1");
        assert_eq!(format!("{}", Digit::D5), "5");
        assert_eq!(format!("{}", Digit::D9), "9");
    }

    #[test]
    fn test_from_digit_to_u8() {
        let digit = Digit::D5;
        let value: u8 = digit.into();
        assert_eq!(value, 5);
    }
}
