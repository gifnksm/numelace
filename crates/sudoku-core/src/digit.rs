//! Sudoku digit representation.

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
    /// The digit 1.
    D1 = 1,
    /// The digit 2.
    D2 = 2,
    /// The digit 3.
    D3 = 3,
    /// The digit 4.
    D4 = 4,
    /// The digit 5.
    D5 = 5,
    /// The digit 6.
    D6 = 6,
    /// The digit 7.
    D7 = 7,
    /// The digit 8.
    D8 = 8,
    /// The digit 9.
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
    fn test_basic_operations() {
        // from_value and value() round-trip for boundary values
        assert_eq!(Digit::from_value(1), Digit::D1);
        assert_eq!(Digit::from_value(9), Digit::D9);
        assert_eq!(Digit::D1.value(), 1);
        assert_eq!(Digit::D9.value(), 9);

        // ALL constant contains all 9 digits in order
        assert_eq!(Digit::ALL.len(), 9);
        assert_eq!(Digit::ALL[0], Digit::D1);
        assert_eq!(Digit::ALL[8], Digit::D9);

        // from_value/value round-trip for all digits
        for digit in Digit::ALL {
            let value = digit.value();
            assert_eq!(Digit::from_value(value), digit);
        }

        // Display trait
        assert_eq!(format!("{}", Digit::D1), "1");
        assert_eq!(format!("{}", Digit::D9), "9");

        // From<Digit> for u8
        let value: u8 = Digit::D5.into();
        assert_eq!(value, 5);
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
}
