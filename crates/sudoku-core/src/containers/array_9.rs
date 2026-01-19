//! Semantically-indexed 9-element array.
//!
//! Provides [`Array9`], a 9-element array with type-safe indexing via
//! [`Index9Semantics`].
//!
//! [`Index9Semantics`]: crate::index::Index9Semantics

use std::{
    array,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Index, IndexMut},
    slice,
};

use crate::index::Index9Semantics;

/// A 9-element array with semantic indexing.
///
/// This type wraps a `[T; 9]` array and provides indexing via semantic values
/// defined by the `S` type parameter, which must implement [`Index9Semantics`].
/// This allows natural domain-specific indexing, such as using digits 1-9 instead
/// of raw indices 0-8.
///
/// # Type Parameters
///
/// * `T` - The element type stored in the array
/// * `S` - The semantics implementation that defines how values map to indices
///
/// # Examples
///
/// ```
/// use sudoku_core::containers::Array9;
/// use sudoku_core::index::DigitSemantics;
///
/// // Create an array of integers indexed by digits 1-9
/// let mut counts: Array9<i32, DigitSemantics> = Array9::from([0; 9]);
///
/// // Use semantic indexing (digit 1-9)
/// counts[1] = 10; // digit 1
/// counts[9] = 20; // digit 9
///
/// assert_eq!(counts[1], 10);
/// assert_eq!(counts[9], 20);
/// ```
pub struct Array9<T, S>
where
    S: Index9Semantics,
{
    array: [T; 9],
    _marker: PhantomData<S>,
}

impl<T, S> Copy for Array9<T, S>
where
    T: Copy,
    S: Index9Semantics,
{
}

impl<T, S> Clone for Array9<T, S>
where
    T: Clone,
    S: Index9Semantics,
{
    fn clone(&self) -> Self {
        Self::from_array(self.array.clone())
    }
}

impl<T, S> PartialEq for Array9<T, S>
where
    T: PartialEq,
    S: Index9Semantics,
{
    fn eq(&self, other: &Self) -> bool {
        self.array == other.array
    }
}

impl<T, S> Eq for Array9<T, S>
where
    T: PartialEq,
    S: Index9Semantics,
{
}

impl<T, S> Hash for Array9<T, S>
where
    T: Hash,
    S: Index9Semantics,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.array.hash(state);
    }
}

impl<T, S> Default for Array9<T, S>
where
    T: Default,
    S: Index9Semantics,
{
    fn default() -> Self {
        Self::from_array(Default::default())
    }
}

impl<T, S> From<[T; 9]> for Array9<T, S>
where
    S: Index9Semantics,
{
    fn from(array: [T; 9]) -> Self {
        Self::from_array(array)
    }
}

impl<T, S> Debug for Array9<T, S>
where
    T: Debug,
    S: Index9Semantics,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.array, f)
    }
}

impl<T, S> Array9<T, S>
where
    S: Index9Semantics,
{
    /// The length of the array (always 9).
    pub const LEN: usize = 9;

    const fn from_array(array: [T; 9]) -> Self {
        Self {
            array,
            _marker: PhantomData,
        }
    }

    /// Returns an iterator over the array elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::containers::Array9;
    /// use sudoku_core::index::DigitSemantics;
    ///
    /// let array: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// let sum: i32 = array.iter().sum();
    /// assert_eq!(sum, 45);
    /// ```
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.array.iter()
    }

    /// Returns a mutable iterator over the array elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_core::containers::Array9;
    /// use sudoku_core::index::DigitSemantics;
    ///
    /// let mut array: Array9<i32, DigitSemantics> = Array9::from([0; 9]);
    /// for elem in array.iter_mut() {
    ///     *elem = 42;
    /// }
    /// assert_eq!(array[1], 42);
    /// ```
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.array.iter_mut()
    }
}

impl<T, S> Index<S::Value> for Array9<T, S>
where
    S: Index9Semantics,
{
    type Output = T;

    fn index(&self, value: S::Value) -> &Self::Output {
        let index = usize::from(S::to_index(value).index());
        &self.array[index]
    }
}

impl<T, S> IndexMut<S::Value> for Array9<T, S>
where
    S: Index9Semantics,
{
    fn index_mut(&mut self, value: S::Value) -> &mut Self::Output {
        let index = usize::from(S::to_index(value).index());
        &mut self.array[index]
    }
}

impl<'a, T, S> IntoIterator for &'a Array9<T, S>
where
    S: Index9Semantics,
{
    type Item = &'a T;

    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, S> IntoIterator for &'a mut Array9<T, S>
where
    S: Index9Semantics,
{
    type Item = &'a mut T;

    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, S> IntoIterator for Array9<T, S>
where
    S: Index9Semantics,
{
    type Item = T;
    type IntoIter = array::IntoIter<T, 9>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{CellIndexSemantics, DigitSemantics};

    #[test]
    fn test_from_array() {
        let array: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(array[1], 1);
        assert_eq!(array[9], 9);
    }

    #[test]
    fn test_index_digit_semantics() {
        let array: Array9<i32, DigitSemantics> = Array9::from([10, 20, 30, 40, 50, 60, 70, 80, 90]);
        // Digit 1 maps to index 0 (value 10)
        assert_eq!(array[1], 10);
        // Digit 5 maps to index 4 (value 50)
        assert_eq!(array[5], 50);
        // Digit 9 maps to index 8 (value 90)
        assert_eq!(array[9], 90);
    }

    #[test]
    fn test_index_cell_semantics() {
        let array: Array9<i32, CellIndexSemantics> = Array9::from([0, 1, 2, 3, 4, 5, 6, 7, 8]);
        for i in 0..9 {
            assert_eq!(array[i], i32::from(i));
        }
    }

    #[test]
    fn test_index_mut() {
        let mut array: Array9<i32, DigitSemantics> = Array9::from([0; 9]);
        array[1] = 100;
        array[5] = 500;
        array[9] = 900;

        assert_eq!(array[1], 100);
        assert_eq!(array[5], 500);
        assert_eq!(array[9], 900);
    }

    #[test]
    fn test_iter() {
        let array: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let sum: i32 = (&array).into_iter().sum();
        assert_eq!(sum, 45);
    }

    #[test]
    #[expect(clippy::manual_slice_fill)]
    fn test_iter_mut() {
        let mut array: Array9<i32, DigitSemantics> = Array9::from([0; 9]);
        for elem in &mut array {
            *elem = 42;
        }
        for i in 1..=9 {
            assert_eq!(array[i], 42);
        }
    }

    #[test]
    fn test_into_iter() {
        let array: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let vec: Vec<i32> = array.into_iter().collect();
        assert_eq!(vec, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_clone() {
        let array1: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let array2 = array1;
        assert_eq!(array1, array2);
    }

    #[test]
    fn test_default() {
        let array: Array9<i32, DigitSemantics> = Array9::default();
        for i in 1..=9 {
            assert_eq!(array[i], 0);
        }
    }

    #[test]
    fn test_eq() {
        let array1: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let array2: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let array3: Array9<i32, DigitSemantics> = Array9::from([9, 8, 7, 6, 5, 4, 3, 2, 1]);

        assert_eq!(array1, array2);
        assert_ne!(array1, array3);
    }

    #[test]
    fn test_debug() {
        let array: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let debug_str = format!("{array:?}");
        assert!(debug_str.contains('1'));
        assert!(debug_str.contains('9'));
    }

    #[test]
    fn test_for_loop() {
        let array: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let mut sum = 0;
        for &elem in &array {
            sum += elem;
        }
        assert_eq!(sum, 45);
    }

    #[test]
    fn test_for_loop_mut() {
        let mut array: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        for elem in &mut array {
            *elem *= 2;
        }
        assert_eq!(array[1], 2);
        assert_eq!(array[5], 10);
        assert_eq!(array[9], 18);
    }

    #[test]
    #[should_panic(expected = "Number must be between 1 and 9")]
    fn test_digit_semantics_panics_on_zero() {
        let array: Array9<i32, DigitSemantics> = Array9::from([0; 9]);
        let _ = array[0]; // Should panic: digit must be 1-9
    }

    #[test]
    #[should_panic(expected = "Number must be between 1 and 9")]
    fn test_digit_semantics_panics_on_ten() {
        let array: Array9<i32, DigitSemantics> = Array9::from([0; 9]);
        let _ = array[10]; // Should panic: digit must be 1-9
    }
}
