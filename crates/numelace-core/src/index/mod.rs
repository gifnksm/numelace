//! Index types and semantics for containers.
//!
//! This module provides index types and their associated semantics for working with
//! 9-element and 81-element containers. These types enable type-safe indexing into
//! arrays and bitsets used throughout the sudoku solver.
//!
//! # Index Types
//!
//! - [`Index9`] - Index into 9-element containers (range 0-8)
//! - [`Index81`] - Index into 81-element containers (board positions in row-major order)
//!
//! # Semantics
//!
//! Semantics types implement the conversion logic between values and indices:
//!
//! - [`DigitSemantics`] - Maps digits 1-9 to indices 0-8
//! - [`CellIndexSemantics`] - Direct 0-8 mapping for generic cell indices
//! - [`PositionSemantics`] - Maps [`Position`] to board indices
//!
//! [`Position`]: crate::Position
//!
//! # Examples
//!
//! ## Using [`Index9`] with [`DigitSemantics`]
//!
//! ```
//! use numelace_core::{
//!     Digit,
//!     index::{DigitSemantics, Index9, Index9Semantics},
//! };
//!
//! // Use digit semantics to convert digit 5 to index
//! let idx = DigitSemantics::to_index(Digit::D5);
//! assert_eq!(idx.index(), 4); // digit 5 maps to index 4
//!
//! // Iterate over all indices
//! let indices: Vec<_> = Index9::all().collect();
//! assert_eq!(indices.len(), 9);
//! ```
//!
//! ## Using [`Index81`] with [`PositionSemantics`]
//!
//! ```
//! use numelace_core::{
//!     Position,
//!     index::{Index81, Index81Semantics, PositionSemantics},
//! };
//!
//! // Convert a position to an index
//! let pos = Position::new(4, 4);
//! let idx = PositionSemantics::to_index(pos);
//! assert_eq!(idx.index(), 40); // row 4, column 4 -> 4*9 + 4
//!
//! // Convert back
//! let pos2 = PositionSemantics::from_index(idx);
//! assert_eq!(pos, pos2);
//! ```
//!
//! ## Implementing custom semantics
//!
//! ```
//! use numelace_core::index::{Index9, Index9Semantics};
//!
//! // Define semantics that map digits 1-9 to indices 0-8
//! struct MyDigitSemantics;
//!
//! impl Index9Semantics for MyDigitSemantics {
//!     type Value = u8;
//!
//!     fn to_index(value: u8) -> Index9 {
//!         assert!((1..=9).contains(&value));
//!         Index9::new(value - 1)
//!     }
//!
//!     fn from_index(index: Index9) -> u8 {
//!         index.index() + 1
//!     }
//! }
//!
//! let idx = MyDigitSemantics::to_index(5);
//! assert_eq!(idx.index(), 4);
//! ```

pub use self::{index_9::*, index_81::*};

mod index_81;
mod index_9;
