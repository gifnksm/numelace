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
        print_puzzle(&puzzle, &stats, None, &[]);
        return;
    }

    let max_tries = args.max_tries;
    if max_tries == 0 {
        eprintln!("--max-tries must be at least 1.");
        process::exit(1);
    }

    let mut best = None;
    for _ in 0..max_tries {
        let puzzle = generator.generate();
        let stats = solve_stats(&solver, &puzzle);
        let score = techniques_score(&stats, &args.techniques);
        match &best {
            None => best = Some((puzzle, stats, score)),
            Some((_, _, best_score)) if score > *best_score => {
                best = Some((puzzle, stats, score));
            }
            _ => {}
        }
    }

    if let Some((puzzle, stats, score)) = best {
        print_puzzle(&puzzle, &stats, Some((max_tries, score)), &args.techniques);
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

fn techniques_score(stats: &TechniqueSolverStats, techniques: &[String]) -> usize {
    techniques
        .iter()
        .map(|name| technique_count(stats, name))
        .sum()
}

fn technique_count(stats: &TechniqueSolverStats, name: &str) -> usize {
    stats
        .applications()
        .iter()
        .find_map(|(technique, count)| {
            if technique.eq_ignore_ascii_case(name) {
                Some(*count)
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn print_puzzle(
    puzzle: &GeneratedPuzzle,
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
    println!("{:#}", puzzle.problem);
    println!();
    println!("Solution:");
    println!("{:#}", puzzle.solution);
    println!();

    println!("Stats:");
    for (name, count) in stats.applications() {
        println!("  {name}: {count}");
    }
    println!("  total: {}", stats.total_steps());
}
