//! Generic container implementations for sudoku data structures.
//!
//! This module provides efficient, type-safe container implementations that are
//! parameterized by index semantics. These containers form the foundation for
//! higher-level sudoku types.
//!
//! # Container Types
//!
//! - [`BitSet9`] - A 9-bit set for representing subsets of 9 elements (e.g., digits 1-9,
//!   cells in a house). Uses a 16-bit integer for efficient storage and fast operations.
//! - [`BitSet81`] - An 81-bit set for representing subsets of board positions.
//!   Uses a 128-bit integer internally for efficient bitwise operations.
//! - [`Array9`] - A 9-element array with type-safe indexing based on semantics.
//!   Provides O(1) access with compile-time guarantees about valid indices.
//!
//! # Semantics Parameterization
//!
//! All containers are generic over semantics types from the [`index`] module
//! (such as [`Index9Semantics`] and [`Index81Semantics`]),
//! which define how values map to indices. This enables:
//!
//! - Type safety: Different semantic types prevent mixing incompatible indices
//! - Clarity: The type signature documents what the container represents
//! - Efficiency: Zero-cost abstraction with no runtime overhead
//!
//! [`index`]: crate::index
//! [`Index9Semantics`]: crate::index::Index9Semantics
//! [`Index81Semantics`]: crate::index::Index81Semantics
//!
//! # Examples
//!
//! ## Using [`BitSet9`] with [`DigitSemantics`]
//!
//! ```
//! use sudoku_core::DigitCandidates;
//!
//! let mut candidates = DigitCandidates::new();
//! candidates.insert(1);
//! candidates.insert(5);
//! candidates.insert(9);
//!
//! assert_eq!(candidates.len(), 3);
//! assert!(candidates.contains(5));
//! assert!(!candidates.contains(2));
//!
//! // Remove a candidate
//! candidates.remove(5);
//! assert_eq!(candidates.len(), 2);
//! ```
//!
//! ## Using [`BitSet81`] with [`PositionSemantics`]
//!
//! ```
//! use sudoku_core::Position;
//! use sudoku_core::candidate_board::DigitPositions;
//!
//! let mut positions = DigitPositions::new();
//! positions.insert(Position::new(0, 0));
//! positions.insert(Position::new(4, 4));
//! positions.insert(Position::new(8, 8));
//!
//! assert_eq!(positions.len(), 3);
//! assert!(positions.contains(Position::new(4, 4)));
//! assert!(!positions.contains(Position::new(3, 3)));
//! ```
//!
//! ## Using [`Array9`] with [`DigitSemantics`]
//!
//! [`DigitSemantics`]: crate::index::DigitSemantics
//! [`PositionSemantics`]: crate::index::PositionSemantics
//!
//! ```
//! use sudoku_core::candidate_board::DigitPositions;
//! use sudoku_core::containers::Array9;
//! use sudoku_core::index::DigitSemantics;
//!
//! // Create an array indexed by digits 1-9
//! let mut digit_array: Array9<DigitPositions, DigitSemantics> =
//!     Array9::from([DigitPositions::FULL; 9]);
//!
//! // Access by digit (1-9), not index (0-8)
//! let positions = &digit_array[5]; // digit 5
//! assert_eq!(positions.len(), 81); // All positions initially
//! ```

pub use self::{array_9::*, bit_set_9::*, bit_set_81::*};

mod array_9;
mod bit_set_81;
mod bit_set_9;
