//! Sudoku solver with technique-based solving and backtracking.
//!
//! This crate provides two complementary solvers:
//!
//! - [`TechniqueSolver`]: Applies human-like solving techniques only
//! - [`BacktrackSolver`]: Combines techniques with backtracking for complete solving
//!
//! # Architecture
//!
//! ## Two-Layer Design
//!
//! The solver uses a two-layer architecture:
//!
//! 1. **[`TechniqueSolver`]**: Applies only human-like techniques, no backtracking
//!    - Returns when stuck (no more progress possible)
//!    - Useful for evaluating puzzle difficulty
//!    - Can be used for step-by-step solving with user hints
//!
//! 2. **[`BacktrackSolver`]**: Uses [`TechniqueSolver`] first, then backtracks when stuck
//!    - Guarantees finding all solutions if they exist
//!    - Enumerates multiple solutions (useful for puzzle validation)
//!    - Suitable for puzzle generation
//!
//! This separation allows testing technique-only solving, evaluating puzzle difficulty,
//! and generating puzzles with specific technique requirements.
//!
//! ## Progress Strategy
//!
//! When any technique makes progress (places a digit or removes a candidate), the solver
//! resets to the first technique. This ensures simpler techniques are always preferred:
//!
//! - **Progress made** → Reset to first technique
//! - **No progress** → Try next technique
//! - **All techniques exhausted** → Stuck (or solved)
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use sudoku_core::CandidateGrid;
//! use sudoku_solver::BacktrackSolver;
//!
//! let solver = BacktrackSolver::with_all_techniques();
//! let grid = CandidateGrid::new();
//!
//! // Get first solution
//! if let Some((solution, stats)) = solver.solve(grid)?.next() {
//!     println!("Solved!");
//!     println!("Assumptions made: {}", stats.assumptions().len());
//!     println!("Technique steps: {}", stats.technique().total_steps());
//!
//!     if stats.solved_without_assumptions() {
//!         println!("No backtracking needed!");
//!     }
//! }
//! # Ok::<(), sudoku_solver::SolverError>(())
//! ```
//!
//! ## Technique-Only Solving
//!
//! ```
//! use sudoku_core::CandidateGrid;
//! use sudoku_solver::TechniqueSolver;
//!
//! let solver = TechniqueSolver::with_all_techniques();
//! let mut grid = CandidateGrid::new();
//!
//! let (solved, stats) = solver.solve(&mut grid)?;
//! if solved {
//!     println!("Solved with techniques only!");
//! } else {
//!     println!("Stuck after {} steps", stats.total_steps());
//!     println!("This puzzle requires backtracking or advanced techniques");
//! }
//! # Ok::<(), sudoku_solver::SolverError>(())
//! ```
//!
//! ## Step-by-Step Solving
//!
//! ```
//! use sudoku_core::CandidateGrid;
//! use sudoku_solver::{TechniqueSolver, TechniqueSolverStats};
//!
//! let solver = TechniqueSolver::with_all_techniques();
//! let mut grid = CandidateGrid::new();
//! let mut stats = TechniqueSolverStats::new();
//!
//! // Apply one technique at a time
//! while solver.step(&mut grid, &mut stats)? {
//!     println!("Progress! Total steps: {}", stats.total_steps());
//!
//!     if grid.is_solved()? {
//!         println!("Puzzle solved!");
//!         break;
//!     }
//! }
//! # Ok::<(), sudoku_solver::SolverError>(())
//! ```
//!
//! ## Checking for Multiple Solutions
//!
//! ```
//! use sudoku_core::CandidateGrid;
//! use sudoku_solver::BacktrackSolver;
//!
//! let solver = BacktrackSolver::with_all_techniques();
//! let grid = CandidateGrid::new();
//!
//! // Check if puzzle has a unique solution
//! let solutions: Vec<_> = solver.solve(grid)?.take(2).collect();
//! match solutions.len() {
//!     0 => println!("No solution"),
//!     1 => println!("Unique solution - valid puzzle"),
//!     _ => println!("Multiple solutions - invalid puzzle"),
//! }
//! # Ok::<(), sudoku_solver::SolverError>(())
//! ```
//!
//! ## Custom Technique Selection
//!
//! ```
//! use sudoku_core::CandidateGrid;
//! use sudoku_solver::{BacktrackSolver, technique::{BoxedTechnique, NakedSingle}};
//!
//! // Use only specific techniques
//! let techniques: Vec<BoxedTechnique> = vec![
//!     Box::new(NakedSingle::new()),
//! ];
//! let solver = BacktrackSolver::with_techniques(techniques);
//!
//! let grid = CandidateGrid::new();
//! if let Some((solution, _)) = solver.solve(grid)?.next() {
//!     println!("Solved!");
//! }
//! # Ok::<(), sudoku_solver::SolverError>(())
//! ```
//!
//! # Available Techniques
//!
//! Currently implemented techniques (in order of difficulty):
//!
//! - [`NakedSingle`](technique::NakedSingle): A cell with only one candidate
//! - [`HiddenSingle`](technique::HiddenSingle): A digit that can only go in one cell in a house
//!
//! # Adding New Techniques
//!
//! To add a new technique:
//!
//! 1. Implement the [`Technique`](technique::Technique) trait:
//!
//! ```
//! use sudoku_core::CandidateGrid;
//! use sudoku_solver::{SolverError, technique::Technique};
//!
//! #[derive(Debug, Clone)]
//! struct MyTechnique;
//!
//! impl Technique for MyTechnique {
//!     fn name(&self) -> &'static str {
//!         "my technique"
//!     }
//!
//!     fn clone_box(&self) -> Box<dyn Technique> {
//!         Box::new(self.clone())
//!     }
//!
//!     fn apply(&self, grid: &mut CandidateGrid) -> Result<bool, SolverError> {
//!         // Apply your technique logic here
//!         // Return Ok(true) if progress was made
//!         Ok(false)
//!     }
//! }
//! ```
//!
//! 2. Add it to the technique list in [`technique::all_techniques()`]
//!
//! 3. Add comprehensive tests in the technique's module
//!
//! # Performance Characteristics
//!
//! - **[`TechniqueSolver`]**: O(n × t) where n is puzzle complexity and t is number of techniques
//! - **[`BacktrackSolver`]**: Worst case O(9^m) where m is number of empty cells
//!   - In practice, much faster due to technique-based pruning
//!   - Grid cloning cost: 144 bytes per assumption (acceptable)
//!
//! # Error Handling
//!
//! The solver returns [`SolverError::Contradiction`] when it detects an invalid state:
//!
//! - A cell with no remaining candidates
//! - Contradictory placements
//!
//! This typically indicates the input puzzle is invalid or unsolvable.
//!
pub use self::{backtrack_solver::*, error::*, technique_solver::*};

mod backtrack_solver;
mod error;
pub mod technique;
mod technique_solver;

#[cfg(test)]
mod testing;
