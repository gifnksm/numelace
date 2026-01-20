//! Core data structures for sudoku applications.
//!
//! This crate provides fundamental, efficient data structures for representing and
//! manipulating sudoku puzzles. These structures are used across solving, generation,
//! and game management components.
//!
//! # Architecture Overview
//!
//! The crate follows a **two-grid architecture** that separates concerns:
//!
//! - [`CandidateGrid`] - Digit-centric interface for solving algorithms
//! - [`DigitGrid`] - Cell-centric interface for simple data access
//!
//! This separation allows each type to provide the most natural interface for its use case.
//!
//! # Core Types
//!
//! ## Basic Sudoku Types
//!
//! - [`Digit`] - Type-safe representation of sudoku digits 1-9
//! - [`Position`] - Grid position with (x, y) coordinates in the range 0-8
//!   - Provides box calculation utilities (`box_index()`, `box_cell_index()`)
//!   - Supports conversion between linear and box-relative coordinates
//!
//! ## Grid Types
//!
//! ### [`CandidateGrid`] (Solving & Constraint Propagation)
//!
//! [`CandidateGrid`] is a candidate tracking grid for solving algorithms:
//!   - Digit-centric representation (tracks where each digit can be placed)
//!   - Automatic constraint propagation and candidate elimination
//!   - Supports technique detection (Hidden Singles, Naked Singles, etc.)
//!
//! ### [`DigitGrid`] (Simple Cell-Centric Interface)
//!
//! [`DigitGrid`] provides an intuitive cell-centric interface:
//!   - Direct "what's in this cell?" queries
//!   - Simple array-based representation using [`Array81`]
//!   - Natural for human reasoning about puzzle state
//!   - String parsing and formatting support (`FromStr`, `Display`)
//!   - Conversion to/from [`CandidateGrid`]
//!
//! ## Candidate Type Aliases
//!
//! - [`DigitCandidates`] - Set of candidate digits (1-9) for a single cell.
//!   A specialized [`BitSet9`] using [`DigitSemantics`].
//!
//! - [`DigitPositions`] - Set of grid positions where a specific digit can be placed.
//!   A type alias for `BitSet81<PositionSemantics>`.
//!
//! - [`HouseMask`] - Bitmask for candidate positions within a house (row/col/box).
//!   A type alias for `BitSet9<CellIndexSemantics>`.
//!
//! # Generic Infrastructure
//!
//! ## Index Types and Semantics
//!
//! The [`index`] module provides index types and semantics for type-safe container access:
//!
//! - [`Index9`] and [`Index81`] - Index types for 9-element and 81-element containers
//! - [`Index9Semantics`] and [`Index81Semantics`] - Traits defining index semantics
//! - [`DigitSemantics`], [`CellIndexSemantics`], [`PositionSemantics`] - Concrete semantics implementations
//!
//! This ensures compile-time safety by preventing accidental mixing of incompatible index types.
//!
//! ## Generic Containers
//!
//! The [`containers`] module provides generic containers parameterized by semantics:
//!
//! - [`BitSet9`] - Efficient 9-element bitset (u16-based)
//! - [`BitSet81`] - Efficient 81-element bitset (u128-based)
//! - [`Array9`] - 9-element array with semantic indexing
//! - [`Array81`] - 81-element array with semantic indexing
//!
//! [`Index9`]: index::Index9
//! [`Index81`]: index::Index81
//! [`Index9Semantics`]: index::Index9Semantics
//! [`Index81Semantics`]: index::Index81Semantics
//! [`DigitSemantics`]: index::DigitSemantics
//! [`CellIndexSemantics`]: index::CellIndexSemantics
//! [`PositionSemantics`]: index::PositionSemantics
//! [`BitSet9`]: containers::BitSet9
//! [`BitSet81`]: containers::BitSet81
//! [`Array9`]: containers::Array9
//! [`Array81`]: containers::Array81
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use sudoku_core::{Digit, DigitCandidates, Position};
//!
//! // Create a position and digit
//! let pos = Position::new(4, 4);
//! let digit = Digit::D5;
//!
//! // Work with candidate sets
//! let mut candidates = DigitCandidates::FULL;
//! candidates.remove(Digit::D1);
//! candidates.remove(Digit::D9);
//! assert_eq!(candidates.len(), 7);
//! ```
//!
//! ## [`CandidateGrid`] - Solving and Constraint Propagation
//!
//! ```
//! use sudoku_core::{CandidateGrid, Digit, DigitCandidates, Position};
//!
//! // Create a candidate grid with all positions having all candidates
//! let mut grid = CandidateGrid::new();
//!
//! // Place a digit - automatically updates candidates in row, column, and box
//! grid.place(Position::new(4, 4), Digit::D5);
//!
//! // Check remaining candidates at a position
//! let candidates: DigitCandidates = grid.get_candidates_at(Position::new(4, 5));
//! assert!(!candidates.contains(Digit::D5)); // D5 removed from same column
//!
//! // Check if the puzzle is consistent (no contradictions)
//! assert!(grid.is_consistent());
//! ```
//!
//! ## [`DigitGrid`] - Simple Cell-Centric Interface
//!
//! ```
//! use sudoku_core::{DigitGrid, Digit, Position};
//!
//! // Parse a grid from a string (dots represent empty cells)
//! let grid: DigitGrid = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
//!     .parse()
//!     .unwrap();
//!
//! // Access cells directly
//! assert_eq!(grid[Position::new(0, 0)], Some(Digit::D5));
//! assert_eq!(grid[Position::new(2, 0)], None);
//!
//! // Display the grid (compact or pretty format)
//! println!("{}", grid);      // 81 characters, no newlines
//! println!("{:#}", grid);    // 9 lines with newlines
//! ```
//!
//! ## Design Rationale
//!
//! ### Why Two Grid Types?
//!
//! - **[`CandidateGrid`]**: Digit-centric interface for solving
//!   - Answers "where can digit X go?" efficiently
//!   - Optimized for constraint propagation
//!
//! - **[`DigitGrid`]**: Cell-centric interface for simple access
//!   - Answers "what's in this cell?" naturally
//!   - Intuitive for human reasoning about puzzle state
//!   - Easy string parsing and formatting
//!
//! Each type provides the most natural interface for its access pattern.

mod candidate_grid;
pub mod containers;
mod digit;
mod digit_grid;
pub mod index;
mod position;

// Re-export commonly used types
pub use self::{candidate_grid::*, digit::*, digit_grid::*, position::*};
