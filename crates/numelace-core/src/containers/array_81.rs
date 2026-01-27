//! Semantically-indexed 81-element array.
//!
//! Provides [`Array81`], an 81-element array with type-safe indexing via
//! [`Index81Semantics`].
//!
//! [`Index81Semantics`]: crate::index::Index81Semantics

use std::{
    array,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Index, IndexMut},
    slice,
};

use crate::index::{Index81, Index81Semantics};

/// An 81-element array with semantic indexing.
///
/// This type wraps a `[T; 81]` array and provides indexing via semantic values
/// defined by the `S` type parameter, which must implement [`Index81Semantics`].
/// This allows natural domain-specific indexing, such as using [`Position`] to
/// index cells in a Sudoku grid.
///
/// The `S` parameter provides **compile-time type safety** through the
/// [Semantics Pattern](crate#semantics-pattern-type-safe-indexing), preventing
/// accidental use of incorrect index types.
///
/// [`Position`]: crate::Position
///
/// # Type Parameters
///
/// * `T` - The element type stored in the array
/// * `S` - The semantics implementation that defines how values map to indices
///
/// # Examples
///
/// ```
/// use numelace_core::{Position, containers::Array81, index::PositionSemantics};
///
/// // Create an array of integers indexed by grid positions
/// let mut grid: Array81<i32, PositionSemantics> = Array81::from([0; 81]);
///
/// // Use semantic indexing (Position)
/// let pos = Position::new(0, 0);
/// grid[pos] = 42;
///
/// assert_eq!(grid[pos], 42);
/// ```
///
/// See the [crate-level documentation](crate#semantics-pattern-type-safe-indexing) for details.
pub struct Array81<T, S>
where
    S: Index81Semantics,
{
    array: [T; 81],
    _marker: PhantomData<S>,
}

impl<T, S> Copy for Array81<T, S>
where
    T: Copy,
    S: Index81Semantics,
{
}

impl<T, S> Clone for Array81<T, S>
where
    T: Clone,
    S: Index81Semantics,
{
    fn clone(&self) -> Self {
        Self::from_array(self.array.clone())
    }
}

impl<T, S> PartialEq for Array81<T, S>
where
    T: PartialEq,
    S: Index81Semantics,
{
    fn eq(&self, other: &Self) -> bool {
        self.array == other.array
    }
}

impl<T, S> Eq for Array81<T, S>
where
    T: PartialEq,
    S: Index81Semantics,
{
}

impl<T, S> Hash for Array81<T, S>
where
    T: Hash,
    S: Index81Semantics,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.array.hash(state);
    }
}

impl<T, S> Default for Array81<T, S>
where
    T: Default + Copy,
    S: Index81Semantics,
{
    fn default() -> Self {
        Self::from_array([T::default(); 81])
    }
}

impl<T, S> From<[T; 81]> for Array81<T, S>
where
    S: Index81Semantics,
{
    fn from(array: [T; 81]) -> Self {
        Self::from_array(array)
    }
}

impl<T, S> Debug for Array81<T, S>
where
    T: Debug,
    S: Index81Semantics,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.array, f)
    }
}

impl<T, S> Array81<T, S>
where
    S: Index81Semantics,
{
    /// The length of the array (always 81).
    pub const LEN: usize = 81;

    /// Creates a new `Array81` from a raw array.
    ///
    /// This is a const function that can be used in const contexts.
    ///
    /// # Example
    ///
    /// ```
    /// use numelace_core::{containers::Array81, index::PositionSemantics};
    ///
    /// const MY_ARRAY: Array81<i32, PositionSemantics> = Array81::from_array([42; 81]);
    /// ```
    #[inline]
    pub const fn from_array(array: [T; 81]) -> Self {
        Self {
            array,
            _marker: PhantomData,
        }
    }

    /// Creates a new `Array81` by calling `f` for each semantic value.
    ///
    /// Values are generated in index order (0-80) and converted via
    /// [`Index81Semantics::from_index`].
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_core::{Position, containers::Array81, index::PositionSemantics};
    ///
    /// let array = Array81::<u8, PositionSemantics>::from_fn(|pos| pos.x() + pos.y());
    /// assert_eq!(array[Position::new(0, 0)], 0);
    /// assert_eq!(array[Position::new(8, 8)], 16);
    /// ```
    #[inline]
    #[expect(clippy::missing_panics_doc)]
    pub fn from_fn<F>(mut f: F) -> Self
    where
        F: FnMut(S::Value) -> T,
    {
        Self::from_array(array::from_fn(|i| {
            let index = Index81::new(u8::try_from(i).unwrap());
            let value = S::from_index(index);
            f(value)
        }))
    }

    /// Returns an iterator over the array elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use numelace_core::{containers::Array81, index::PositionSemantics};
    ///
    /// let array: Array81<i32, PositionSemantics> = Array81::from([1; 81]);
    /// let sum: i32 = array.iter().sum();
    /// assert_eq!(sum, 81);
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
    /// use numelace_core::{Position, containers::Array81, index::PositionSemantics};
    ///
    /// let mut array: Array81<i32, PositionSemantics> = Array81::from([0; 81]);
    /// for elem in array.iter_mut() {
    ///     *elem = 42;
    /// }
    /// assert_eq!(array[Position::new(0, 0)], 42);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.array.iter_mut()
    }
}

impl<T, S> Index<S::Value> for Array81<T, S>
where
    S: Index81Semantics,
{
    type Output = T;

    #[inline]
    fn index(&self, value: S::Value) -> &Self::Output {
        let index = usize::from(S::to_index(value).index());
        &self.array[index]
    }
}

impl<T, S> IndexMut<S::Value> for Array81<T, S>
where
    S: Index81Semantics,
{
    #[inline]
    fn index_mut(&mut self, value: S::Value) -> &mut Self::Output {
        let index = usize::from(S::to_index(value).index());
        &mut self.array[index]
    }
}

impl<'a, T, S> IntoIterator for &'a Array81<T, S>
where
    S: Index81Semantics,
{
    type Item = &'a T;

    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, S> IntoIterator for &'a mut Array81<T, S>
where
    S: Index81Semantics,
{
    type Item = &'a mut T;

    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, S> IntoIterator for Array81<T, S>
where
    S: Index81Semantics,
{
    type Item = T;
    type IntoIter = array::IntoIter<T, 81>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Position, index::PositionSemantics};

    #[test]
    fn test_basic_indexing() {
        #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let values = array::from_fn(|i| i as i32);
        let array: Array81<i32, PositionSemantics> = Array81::from(values);

        // Position (x, y) maps to index (y * 9 + x)
        assert_eq!(array[Position::new(0, 0)], 0);
        assert_eq!(array[Position::new(4, 4)], 40);
        assert_eq!(array[Position::new(8, 8)], 80);

        let mut array: Array81<i32, PositionSemantics> = Array81::from([0; 81]);
        array[Position::new(0, 0)] = 100;
        array[Position::new(4, 4)] = 500;
        array[Position::new(8, 8)] = 900;

        assert_eq!(array[Position::new(0, 0)], 100);
        assert_eq!(array[Position::new(4, 4)], 500);
        assert_eq!(array[Position::new(8, 8)], 900);
    }

    #[test]
    fn test_from_fn() {
        let array: Array81<u8, PositionSemantics> =
            Array81::from_fn(|pos: Position| pos.x() + pos.y());
        assert_eq!(array[Position::new(0, 0)], 0);
        assert_eq!(array[Position::new(4, 4)], 8);
        assert_eq!(array[Position::new(8, 8)], 16);
    }

    #[test]
    fn test_iteration() {
        let array: Array81<i32, PositionSemantics> = Array81::from([1; 81]);

        // Immutable iteration
        let sum: i32 = (&array).into_iter().sum();
        assert_eq!(sum, 81);

        // Mutable iteration
        let mut array = array;
        #[expect(clippy::manual_slice_fill)]
        for elem in &mut array {
            *elem = 42;
        }
        assert_eq!(array[Position::new(0, 0)], 42);

        // IntoIter
        let vec: Vec<i32> = array.into_iter().collect();
        assert_eq!(vec.len(), 81);
        assert_eq!(vec[0], 42);
        assert_eq!(vec[80], 42);
    }

    #[test]
    fn test_clone() {
        // Clone properly handles PhantomData marker
        let array1: Array81<i32, PositionSemantics> = Array81::from([42; 81]);
        #[expect(clippy::clone_on_copy)]
        let array2 = array1.clone();
        assert_eq!(array1, array2);
        assert_eq!(array2[Position::new(5, 5)], 42);
    }

    #[test]
    fn test_eq() {
        // Eq ignores PhantomData marker, compares only array content
        let array1: Array81<i32, PositionSemantics> = Array81::from([1; 81]);
        let array2: Array81<i32, PositionSemantics> = Array81::from([1; 81]);
        let array3: Array81<i32, PositionSemantics> = Array81::from([2; 81]);

        assert_eq!(array1, array2);
        assert_ne!(array1, array3);
    }

    #[test]
    fn test_default() {
        // Default properly initializes with PhantomData marker
        let array: Array81<i32, PositionSemantics> = Array81::default();
        assert_eq!(array[Position::new(0, 0)], 0);
        assert_eq!(array[Position::new(8, 8)], 0);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashMap;

        let array1: Array81<i32, PositionSemantics> = Array81::from([1; 81]);
        let array2: Array81<i32, PositionSemantics> = Array81::from([1; 81]);
        let array3: Array81<i32, PositionSemantics> = Array81::from([2; 81]);

        let mut map = HashMap::new();
        map.insert(array1, "first");
        map.insert(array2, "second");
        map.insert(array3, "third");

        // Equal arrays should have same hash
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&array1), Some(&"second"));
        assert_eq!(map.get(&array3), Some(&"third"));
    }
}
