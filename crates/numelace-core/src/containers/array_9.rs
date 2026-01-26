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
/// The `S` parameter provides **compile-time type safety** through the
/// [Semantics Pattern](crate#semantics-pattern-type-safe-indexing), preventing
/// accidental use of incorrect index types.
///
/// # Type Parameters
///
/// * `T` - The element type stored in the array
/// * `S` - The semantics implementation that defines how values map to indices
///
/// # Examples
///
/// ```
/// use numelace_core::{Digit, containers::Array9, index::DigitSemantics};
///
/// // Create an array of integers indexed by digits 1-9
/// let mut counts: Array9<i32, DigitSemantics> = Array9::from([0; 9]);
///
/// // Use semantic indexing (digit 1-9)
/// counts[Digit::D1] = 10; // digit 1
/// counts[Digit::D9] = 20; // digit 9
///
/// assert_eq!(counts[Digit::D1], 10);
/// assert_eq!(counts[Digit::D9], 20);
/// ```
///
/// See the [crate-level documentation](crate#semantics-pattern-type-safe-indexing) for details.
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

    /// Creates a new `Array9` from a raw array.
    ///
    /// This is a const function that can be used in const contexts.
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_core::{containers::Array9, index::CellIndexSemantics};
    ///
    /// const MY_ARRAY: Array9<i32, CellIndexSemantics> = Array9::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// assert_eq!(MY_ARRAY[0], 1);
    /// ```
    #[inline]
    pub const fn from_array(array: [T; 9]) -> Self {
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
    /// use numelace_core::{containers::Array9, index::DigitSemantics};
    ///
    /// let array: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// let sum: i32 = array.iter().sum();
    /// assert_eq!(sum, 45);
    /// ```
    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.array.iter()
    }

    /// Returns a mutable iterator over the array elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_core::{Digit, containers::Array9, index::DigitSemantics};
    ///
    /// let mut array: Array9<i32, DigitSemantics> = Array9::from([0; 9]);
    /// for elem in array.iter_mut() {
    ///     *elem = 42;
    /// }
    /// assert_eq!(array[Digit::D1], 42);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.array.iter_mut()
    }
}

impl<T, S> Index<S::Value> for Array9<T, S>
where
    S: Index9Semantics,
{
    type Output = T;

    #[inline]
    fn index(&self, value: S::Value) -> &Self::Output {
        let index = usize::from(S::to_index(value).index());
        &self.array[index]
    }
}

impl<T, S> IndexMut<S::Value> for Array9<T, S>
where
    S: Index9Semantics,
{
    #[inline]
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
    use crate::{
        digit::Digit::*,
        index::{CellIndexSemantics, DigitSemantics},
    };

    #[test]
    fn test_basic_indexing() {
        let array: Array9<i32, DigitSemantics> = Array9::from([10, 20, 30, 40, 50, 60, 70, 80, 90]);

        // Digit N maps to index (N-1)
        assert_eq!(array[D1], 10);
        assert_eq!(array[D5], 50);
        assert_eq!(array[D9], 90);

        // Cell index semantics
        let array: Array9<i32, CellIndexSemantics> = Array9::from([0, 1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(array[0], 0);
        assert_eq!(array[4], 4);
        assert_eq!(array[8], 8);

        // Mutable indexing
        let mut array: Array9<i32, DigitSemantics> = Array9::from([0; 9]);
        array[D1] = 100;
        array[D5] = 500;
        array[D9] = 900;
        assert_eq!(array[D1], 100);
        assert_eq!(array[D5], 500);
        assert_eq!(array[D9], 900);
    }

    #[test]
    fn test_iteration() {
        let array: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);

        // Immutable iteration
        let sum: i32 = (&array).into_iter().sum();
        assert_eq!(sum, 45);

        // Mutable iteration
        let mut array = array;
        #[expect(clippy::manual_slice_fill)]
        for elem in &mut array {
            *elem = 42;
        }
        assert_eq!(array[D1], 42);

        // IntoIter
        let vec: Vec<i32> = array.into_iter().collect();
        assert_eq!(vec.len(), 9);
        assert_eq!(vec[0], 42);
        assert_eq!(vec[8], 42);
    }

    #[test]
    fn test_clone() {
        // Clone properly handles PhantomData marker
        let array1: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        #[expect(clippy::clone_on_copy)]
        let array2 = array1.clone();
        assert_eq!(array1, array2);
        assert_eq!(array2[D5], 5);
    }

    #[test]
    fn test_eq() {
        // Eq ignores PhantomData marker, compares only array content
        let array1: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let array2: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let array3: Array9<i32, DigitSemantics> = Array9::from([9, 8, 7, 6, 5, 4, 3, 2, 1]);

        assert_eq!(array1, array2);
        assert_ne!(array1, array3);
    }

    #[test]
    fn test_default() {
        // Default properly initializes with PhantomData marker
        let array: Array9<i32, DigitSemantics> = Array9::default();
        assert_eq!(array[D1], 0);
        assert_eq!(array[D9], 0);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashMap;

        // Equal arrays should have same hash
        let array1: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let array2: Array9<i32, DigitSemantics> = Array9::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let array3: Array9<i32, DigitSemantics> = Array9::from([9, 8, 7, 6, 5, 4, 3, 2, 1]);

        let mut map = HashMap::new();
        map.insert(array1, "first");
        map.insert(array2, "second");
        map.insert(array3, "third");

        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&array1), Some(&"second"));
        assert_eq!(map.get(&array3), Some(&"third"));
    }
}
