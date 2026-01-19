//! Core data structures for sudoku applications.
//!
//! This crate provides fundamental, efficient data structures for representing and
//! manipulating sudoku puzzles. These structures are used across solving, generation,
//! and game management components.
//!
//! # Overview
//!
//! The crate is organized around two main concepts:
//!
//! 1. **Index semantics** - Define how values map to indices in containers
//!    - [`index`]: Index types and semantics for both 9-element and 81-element containers,
//!      including [`Index9`], [`Index81`], and semantics types like
//!      [`DigitSemantics`], [`CellIndexSemantics`], and [`PositionSemantics`].
//!
//! 2. **Generic containers** - Containers parameterized by semantics
//!    - [`containers`]: Generic container implementations including [`BitSet9`],
//!      [`BitSet81`], and [`Array9`]. These are parameterized by index semantics
//!      to provide type-safe, efficient data structures.
//!
//! 3. **Specialized types** - Convenient type aliases and higher-level types
//!    - [`digit_candidates`]: Candidate digits (1-9) for a single cell
//!    - [`candidate_board`]: Board-wide candidate tracking
//!    - [`position`]: Board position (x, y) coordinate types
//!
//! [`Index9`]: index::Index9
//! [`Index81`]: index::Index81
//! [`DigitSemantics`]: index::DigitSemantics
//! [`CellIndexSemantics`]: index::CellIndexSemantics
//! [`PositionSemantics`]: index::PositionSemantics
//! [`BitSet9`]: containers::BitSet9
//! [`BitSet81`]: containers::BitSet81
//! [`Array9`]: containers::Array9
//!
//! # Examples
//!
//! ```
//! use sudoku_core::{CandidateBoard, DigitCandidates, Position};
//!
//! // Create a candidate board
//! let mut board = CandidateBoard::new();
//!
//! // Place a digit
//! board.place(Position::new(4, 4), 5);
//!
//! // Check remaining candidates
//! let candidates: DigitCandidates = board.get_candidates_at(Position::new(4, 5));
//! assert!(!candidates.contains(5)); // 5 removed from same column
//! ```

pub mod candidate_board;
pub mod containers;
pub mod digit_candidates;
pub mod index;
pub mod position;

// Re-export commonly used types
pub use self::{
    candidate_board::{CandidateBoard, DigitPositions, HouseMask},
    digit_candidates::DigitCandidates,
    position::Position,
};
