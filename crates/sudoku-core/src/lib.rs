//! Core data structures for sudoku solvers.
//!
//! This crate provides efficient, specialized data structures for working with
//! sudoku puzzles, particularly bitsets and candidate tracking.
//!
//! # Overview
//!
//! The crate is organized around two main concepts:
//!
//! 1. **Index semantics** - Define how values map to indices in containers
//!    - [`index_9`]: Index types and semantics for 9-element containers
//!    - [`index_81`]: Index types and semantics for 81-element containers
//!
//! 2. **Generic containers** - Bitsets parameterized by semantics
//!    - [`bit_set_9`]: Generic 9-bit set implementation
//!    - [`bit_set_81`]: Generic 81-bit set implementation
//!
//! 3. **Specialized types** - Convenient type aliases with specific semantics
//!    - [`digit_candidates`]: Candidate digits (1-9) for a single cell
//!    - [`candidate_board`]: Board-wide candidate tracking
//!    - [`position`]: Board position (x, y) coordinate types
//!
//! # Examples
//!
//! ```
//! use sudoku_core::{CandidateBoard, Position, DigitCandidates};
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

pub mod bit_set_81;
pub mod bit_set_9;
pub mod candidate_board;
pub mod digit_candidates;
pub mod index_81;
pub mod index_9;
pub mod position;

// Re-export commonly used types
pub use self::{
    candidate_board::{CandidateBoard, DigitPositions, HouseMask},
    digit_candidates::DigitCandidates,
    position::Position,
};
