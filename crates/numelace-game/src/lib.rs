//! Game logic for Sudoku gameplay.
//!
//! This crate manages game sessions and player interactions, providing the core
//! game logic that sits between puzzle generation (numelace-generator) and the
//! user interface (numelace-app).
//!
//! # Overview
//!
//! The [`Game`] struct represents a single Sudoku game session, tracking:
//! - Initial puzzle cells (given/fixed cells that cannot be modified)
//! - Player-filled cells (editable by the player)
//! - Empty cells (not yet filled)
//!
//! # Design
//!
//! ## Design Decisions
//!
//! - **Permissive validation**: Allows rule-violating inputs (e.g., duplicate digits).
//!   Players can experiment freely, and mistakes are discovered organically.
//! - **Strict rule checks (optional)**: Operations accept a [`RuleCheckPolicy`]; `Strict`
//!   rejects inputs that would violate Sudoku rules, while `Permissive` allows them.
//! - **Completion detection**: A game is considered solved when all cells are filled
//!   and there are no rule violations (accepts any valid solution).
//! - **Cell state tracking**: Uses [`CellState`] enum to distinguish between given,
//!   filled, and empty cells at the type level.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use numelace_core::{Digit, Position};
//! use numelace_game::{CellState, Game, InputDigitOptions};
//! use numelace_generator::PuzzleGenerator;
//! use numelace_solver::TechniqueSolver;
//!
//! // Generate a puzzle
//! let solver = TechniqueSolver::with_all_techniques();
//! let generator = PuzzleGenerator::new(&solver);
//! let puzzle = generator.generate();
//!
//! // Start a new game
//! let mut game = Game::new(puzzle);
//!
//! // Find an empty cell
//! let empty_pos = *Position::ALL
//!     .iter()
//!     .find(|&&pos| game.cell(pos).is_empty())
//!     .expect("puzzle has empty cells");
//!
//! // Fill it with a digit
//! game.set_digit(empty_pos, Digit::D5, &InputDigitOptions::default())
//!     .unwrap();
//!
//! // Check if solved
//! if game.is_solved() {
//!     println!("Puzzle completed!");
//! }
//! ```

mod cell_state;
mod error;
mod game;
mod input;

pub use cell_state::CellState;
pub use error::GameError;
pub use game::Game;
pub use input::{
    InputBlockReason, InputDigitOptions, InputOperation, NoteCleanupPolicy, RuleCheckPolicy,
};
