use super::{Array9, Array81};
use crate::index::{CellIndexSemantics, DigitSemantics, PositionSemantics};

/// A 9-element array indexed by [`Digit`](crate::Digit) via [`DigitSemantics`].
pub type DigitIndexedArray<T> = Array9<T, DigitSemantics>;

/// A 9-element array indexed by house cell indices (0-8) via [`CellIndexSemantics`].
pub type CellIndexIndexedArray<T> = Array9<T, CellIndexSemantics>;

/// An 81-element array indexed by [`Position`](crate::Position) via [`PositionSemantics`].
pub type PositionIndexedArray<T> = Array81<T, PositionSemantics>;
