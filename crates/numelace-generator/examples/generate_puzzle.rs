//! Example demonstrating basic Sudoku puzzle generation.
//!
//! This example shows how to:
//! - Create a `PuzzleGenerator` with a `TechniqueSolver`
//! - Generate a random puzzle
//! - Display the puzzle, solution, and seed
//!
//! # Usage
//!
//! ```sh
//! cargo run --example generate_puzzle
//! ```

use numelace_generator::PuzzleGenerator;
use numelace_solver::{TechniqueSolver, technique::TechniqueGrid};

fn main() {
    let solver = TechniqueSolver::with_all_techniques();
    let generator = PuzzleGenerator::new(&solver);

    let puzzle = generator.generate();
    println!("Seed:");
    println!("  {}", puzzle.seed);
    println!();
    println!("Problem:");
    println!("{:#}", puzzle.problem);
    println!();
    println!("Solution:");
    println!("{:#}", puzzle.solution);
    println!();

    let mut grid = TechniqueGrid::from(puzzle.problem);
    let (is_solved, stats) = solver.solve(&mut grid).unwrap();
    assert!(is_solved);
    println!("Stats:");
    for (name, count) in stats.applications() {
        println!("  {name}: {count}");
    }
    println!("  total: {}", stats.total_steps());
}
