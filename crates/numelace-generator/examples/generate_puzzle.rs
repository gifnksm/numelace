//! Example demonstrating basic Sudoku puzzle generation.
//!
//! This example shows how to:
//! - Create a `PuzzleGenerator` with a `TechniqueSolver`
//! - Generate a random puzzle
//! - Display the puzzle, solution, and seed
//! - Filter puzzles by technique usage counts
//!
//! # Usage
//!
//! ```sh
//! cargo run --example generate_puzzle
//! ```
//!
//! Filter for puzzles by selecting the one that maximizes the total count of the
//! specified techniques within the sampling budget:
//!
//! ```sh
//! cargo run --example generate_puzzle -- --technique "Locked Candidates"
//! ```
//!
//! Control the sampling budget (default: 10000):
//!
//! ```sh
//! cargo run --example generate_puzzle -- --technique "Locked Candidates" --max-tries 10000
//! ```
//!
//! Select the solver technique set (fundamental or basic):
//!
//! ```sh
//! cargo run --example generate_puzzle -- --solver fundamental
//! ```
//!
//! Multiple techniques can be required (case-insensitive):
//!
//! ```sh
//! cargo run --example generate_puzzle -- --technique "Locked Candidates" --technique "Hidden Single"
//! ```

use std::process;

use clap::{Parser, ValueEnum};
use numelace_generator::{GeneratedPuzzle, PuzzleGenerator};
use numelace_solver::{TechniqueGrid, TechniqueSolver, TechniqueSolverStats, technique};
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SolverKind {
    All,
    Fundamental,
    Basic,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Solver technique set to use for generation and scoring.
    #[arg(long, value_name = "KIND", default_value = "all")]
    solver: SolverKind,

    /// Technique name to require in stats (case-insensitive). Repeatable.
    #[arg(short, long = "technique", value_name = "TECHNIQUE", num_args = 1..)]
    techniques: Vec<String>,

    /// Maximum puzzles to sample when filtering.
    #[arg(long, value_name = "COUNT", default_value_t = 10_000)]
    max_tries: usize,
}

fn main() {
    let args = Args::parse();
    let (solver, available) = build_solver(args.solver);
    let generator = PuzzleGenerator::new(&solver);

    if !args.techniques.is_empty() {
        let unknown = args
            .techniques
            .iter()
            .filter(|name| !technique_name_matches(name, &available))
            .cloned()
            .collect::<Vec<_>>();
        if !unknown.is_empty() {
            eprintln!("Unknown technique(s): {}", unknown.join(", "));
            eprintln!("Available techniques:");
            for name in &available {
                eprintln!("  {name}");
            }
            process::exit(2);
        }
    }

    if args.techniques.is_empty() {
        let puzzle = generator.generate();
        let stats = solve_stats(&solver, &puzzle);
        print_puzzle(&puzzle, &solver, &stats, None, &[]);
        return;
    }

    let max_tries = args.max_tries;
    if max_tries == 0 {
        eprintln!("--max-tries must be at least 1.");
        process::exit(1);
    }

    let best = (0..max_tries)
        .into_par_iter()
        .map(|_| {
            let puzzle = generator.generate();
            let stats = solve_stats(&solver, &puzzle);
            let score = techniques_score(&solver, &stats, &args.techniques);
            (puzzle, stats, score)
        })
        .max_by(|a, b| a.2.cmp(&b.2));

    if let Some((puzzle, stats, score)) = best {
        print_puzzle(
            &puzzle,
            &solver,
            &stats,
            Some((max_tries, score)),
            &args.techniques,
        );
        return;
    }

    eprintln!("No puzzle matched the requested techniques.");
    process::exit(1);
}

fn build_solver(kind: SolverKind) -> (TechniqueSolver, Vec<&'static str>) {
    let techniques = match kind {
        SolverKind::All => technique::all_techniques(),
        SolverKind::Fundamental => technique::fundamental_techniques(),
        SolverKind::Basic => technique::basic_techniques(),
    };
    let names = techniques
        .iter()
        .map(|technique| technique.name())
        .collect();
    (TechniqueSolver::new(techniques), names)
}

fn technique_name_matches(name: &str, available: &[&'static str]) -> bool {
    available
        .iter()
        .any(|available| available.eq_ignore_ascii_case(name))
}

fn solve_stats(solver: &TechniqueSolver, puzzle: &GeneratedPuzzle) -> TechniqueSolverStats {
    let mut grid = TechniqueGrid::from(puzzle.problem.clone());
    let (is_solved, stats) = solver.solve(&mut grid).unwrap();
    assert!(is_solved);
    stats
}

fn techniques_score(
    solver: &TechniqueSolver,
    stats: &TechniqueSolverStats,
    techniques: &[String],
) -> usize {
    techniques
        .iter()
        .map(|name| technique_count(solver, stats, name))
        .sum()
}

fn technique_count(solver: &TechniqueSolver, stats: &TechniqueSolverStats, name: &str) -> usize {
    let Some(i) = solver
        .techniques()
        .iter()
        .position(|technique| technique.name() == name)
    else {
        return 0;
    };
    stats.applications()[i]
}

fn print_puzzle(
    puzzle: &GeneratedPuzzle,
    solver: &TechniqueSolver,
    stats: &TechniqueSolverStats,
    selection: Option<(usize, usize)>,
    techniques: &[String],
) {
    println!("Seed:");
    println!("  {}", puzzle.seed);
    println!();

    if let Some((max_tries, best_score)) = selection {
        println!("Selection:");
        println!("  Techniques: {}", techniques.join(", "));
        println!("  Max tries: {max_tries}");
        println!("  Best score: {best_score}");
        println!();
    }

    println!("Problem:");
    println!("  {}", puzzle.problem);
    println!();
    println!("Solution:");
    println!("  {}", puzzle.solution);
    println!();

    println!("Stats:");
    for (i, count) in stats.applications().iter().enumerate() {
        let name = solver.techniques()[i].name();
        println!("  {name}: {count}");
    }
    println!("  total: {}", stats.total_steps());
}
