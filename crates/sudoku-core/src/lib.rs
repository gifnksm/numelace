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
//! ### [`CandidateGrid`] (Candidate Tracking)
//!
//! [`CandidateGrid`] tracks which digits can be placed at each position:
//!   - Digit-centric representation optimized for "where can digit X go?" queries
//!   - Provides operations for placing digits and removing candidates
//!   - Used by solving algorithms to implement techniques
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
//! - [`DigitSet`] - A set of sudoku digits (1-9).
//!   A specialized [`BitSet9`] using [`DigitSemantics`].
//!
//! - [`DigitPositions`] - A set of grid positions.
//!   A specialized [`BitSet81`] using [`PositionSemantics`].
//!
//! - [`HouseMask`] - A set of cell indices (0-8) within a house (row, column, or box).
//!   A specialized [`BitSet9`] using [`CellIndexSemantics`].
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
//! # Semantics Pattern: Type-Safe Indexing
//!
//! This crate employs a **Semantics Pattern** throughout its container types to achieve
//! three key goals:
//!
//! 1. **Compile-time type safety** - Prevent mixing incompatible index types
//! 2. **Implementation reuse** - Share generic implementations across different semantics
//! 3. **Efficient data structures** - Provide specialized containers (bitsets, fixed-size arrays)
//!    optimized for sudoku operations
//!
//! ## The Problem
//!
//! In sudoku code, we work with multiple kinds of indices:
//! - **Digits** (1-9) representing sudoku values
//! - **Positions** (x, y coordinates) representing cells on the board
//! - **Cell indices** (0-8) representing positions within a house (row/column/box)
//!
//! Using raw arrays like `[T; 9]` or `[T; 81]` or implementing custom `Index` trait for each
//! type has drawbacks:
//! - Easy to accidentally use a digit index where a position index is expected, or vice versa
//!   (both are `usize`, so the compiler cannot catch this mistake)
//! - Repetitive implementations for similar containers
//! - Cannot leverage efficient bitset representations
//!
//! ## The Solution
//!
//! All container types ([`Array9`], [`Array81`], [`BitSet9`], [`BitSet81`]) are parameterized
//! by a **semantics type** that defines what kind of values can be used as indices or elements:
//!
//! ```
//! # use sudoku_core::{Digit, Position, containers::BitSet9, index::DigitSemantics};
//! // BitSet9 parameterized by DigitSemantics - can only contain Digit elements
//! let mut digit_set: BitSet9<DigitSemantics> = BitSet9::new();
//! digit_set.insert(Digit::D5);  // ✓ Compiles - Digit is the correct type
//!
//! // Position cannot be used as an element for a digit set
//! // digit_set.insert(Position::new(0, 0));  // ✗ Compile error!
//! ```
//!
//! ## Benefits
//!
//! 1. **Type Safety**: You cannot accidentally use a `Digit` to index a
//!    position-based container, or vice versa. The compiler prevents mixing incompatible types.
//!
//! 2. **Implementation Reuse**: Generic implementations are shared across all semantics.
//!    For example, `BitSet9<S>` provides the same set operations for digits, cell indices,
//!    or any other 9-element type, without code duplication.
//!
//! 3. **Efficient Data Structures**: Specialized containers optimized for sudoku:
//!    - `BitSet9` - 16-bit integer for 9-element sets (faster than `HashSet` or `Vec`)
//!    - `BitSet81` - 128-bit integer for 81-element sets (compact position tracking)
//!    - `Array9`/`Array81` - Fixed-size arrays with semantic indexing (no heap allocation)
//!
//! 4. **Self-Documenting Code**: The type signature clearly shows what the container holds.
//!    - `BitSet9<DigitSemantics>` is a set of digits
//!    - `BitSet81<PositionSemantics>` is a set of positions
//!    - `Array81<Option<Digit>, PositionSemantics>` is position-indexed cells
//!
//! 5. **Natural Domain Modeling**: Code uses domain concepts (`Digit`, `Position`) directly
//!    rather than raw integers or implementing custom `Index` traits for each type.
//!
//! ## Common Semantics
//!
//! - [`DigitSemantics`] - Maps [`Digit`] (1-9) to indices (0-8)
//! - [`PositionSemantics`] - Maps [`Position`] to row-major indices (0-80)
//! - [`CellIndexSemantics`] - Direct mapping for indices within a house (0-8)
//!
//! ## Type Aliases for Convenience
//!
//! The crate provides type aliases for common combinations:
//!
//! ```
//! # use sudoku_core::{DigitSet, DigitPositions, HouseMask};
//! // Instead of: BitSet9<DigitSemantics>
//! let candidates: DigitSet = DigitSet::FULL;
//!
//! // Instead of: BitSet81<PositionSemantics>
//! let positions: DigitPositions = DigitPositions::EMPTY;
//!
//! // Instead of: BitSet9<CellIndexSemantics>
//! let mask: HouseMask = HouseMask::new();
//! ```
//!
//! ## Performance
//!
//! The semantics pattern has **zero runtime cost**:
//! - Conversions are simple arithmetic operations (`y * 9 + x`, `digit - 1`)
//! - Small `const fn` functions are inlined by the compiler
//! - The semantics type parameter is zero-sized (`PhantomData`)
//!
//! The type safety benefit far outweighs any theoretical overhead.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use sudoku_core::{Digit, DigitSet, Position};
//!
//! // Create a position and digit
//! let pos = Position::new(4, 4);
//! let digit = Digit::D5;
//!
//! // Work with candidate sets
//! let mut candidates = DigitSet::FULL;
//! candidates.remove(Digit::D1);
//! candidates.remove(Digit::D9);
//! assert_eq!(candidates.len(), 7);
//! ```
//!
//! ## [`CandidateGrid`] - Candidate Tracking
//!
//! ```
//! use sudoku_core::{CandidateGrid, Digit, Position};
//!
//! // Create a candidate grid with all positions having all candidates
//! let mut grid = CandidateGrid::new();
//!
//! // Place a digit at a position
//! grid.place(Position::new(0, 0), Digit::D5);
//!
//! // Query: where can D5 go? (digit-centric)
//! let positions = grid.digit_positions(Digit::D5);
//! assert!(positions.contains(Position::new(0, 0)));
//! assert!(positions.contains(Position::new(1, 1))); // Still available elsewhere
//!
//! // Query: what can go here? (cell-centric)
//! let candidates = grid.candidates_at(Position::new(0, 0));
//! assert_eq!(candidates.len(), 1); // Only D5 at placed cell
//! ```
//!
//! ## [`DigitGrid`] - Simple Cell-Centric Interface
//!
//! ```
//! use sudoku_core::{Digit, DigitGrid, Position};
//!
//! // Parse a grid from a string (dots represent empty cells)
//! let grid: DigitGrid =
//!     "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
//!         .parse()
//!         .unwrap();
//!
//! // Access cells directly
//! assert_eq!(grid[Position::new(0, 0)], Some(Digit::D5));
//! assert_eq!(grid[Position::new(2, 0)], None);
//!
//! // Display the grid (compact or pretty format)
//! println!("{}", grid); // 81 characters, no newlines
//! println!("{:#}", grid); // 9 lines with newlines
//! ```
//!
//! ## Design Rationale
//!
//! ### Why Two Grid Types?
//!
//! - **[`CandidateGrid`]**: Digit-centric interface
//!   - Answers "where can digit X go?" efficiently
//!   - Provides primitives for implementing solving techniques
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
