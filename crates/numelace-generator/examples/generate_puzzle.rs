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
//! Multiple techniques can be required (case-insensitive), including tiers:
//!
//! ```sh
//! cargo run --example generate_puzzle -- --technique "Locked Candidates" --technique intermediate
//! ```

use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZero,
    process,
};

use clap::{Parser, ValueEnum};
use numelace_generator::{GeneratedPuzzle, PuzzleGenerator};
use numelace_solver::{
    TechniqueGrid, TechniqueSolver, TechniqueSolverStats, TechniqueTier, technique,
};
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SolverKind {
    All,
    Fundamental,
    Basic,
}

#[derive(Debug, Clone)]
enum TechniqueSelector {
    Name(String),
    Tier(TechniqueTier),
}

impl TechniqueSelector {
    fn parse(
        input: &str,
        available_names: &[&'static str],
        available_tiers: &BTreeSet<TechniqueTier>,
    ) -> Option<Self> {
        if let Some(tier) = parse_tier(input) {
            return available_tiers
                .contains(&tier)
                .then_some(TechniqueSelector::Tier(tier));
        }

        if technique_name_matches(input, available_names) {
            return Some(TechniqueSelector::Name(input.to_string()));
        }

        None
    }

    fn label(&self) -> String {
        match self {
            TechniqueSelector::Name(name) => name.clone(),
            TechniqueSelector::Tier(tier) => tier_label(*tier).to_string(),
        }
    }

    fn score(&self, solver: &TechniqueSolver, stats: &TechniqueSolverStats) -> usize {
        match self {
            TechniqueSelector::Name(name) => technique_count(solver, stats, name),
            TechniqueSelector::Tier(tier) => technique_tier_count(solver, stats, *tier),
        }
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Solver technique set to use for generation and scoring.
    #[arg(long, value_name = "KIND", default_value = "all")]
    solver: SolverKind,

    /// Technique name or tier to require in stats (case-insensitive). Repeatable.
    #[arg(short, long = "technique", value_name = "TECHNIQUE", num_args = 1..)]
    techniques: Vec<String>,

    /// Maximum puzzles to sample when filtering.
    #[arg(long, value_name = "COUNT", default_value_t = NonZero::new(10_000).unwrap())]
    max_tries: NonZero<usize>,
}

fn main() {
    let args = Args::parse();
    let (solver, available_names, available_tiers) = build_solver(args.solver);
    let generator = PuzzleGenerator::new(&solver);

    let (selectors, unknown) =
        parse_technique_selectors(&args.techniques, &available_names, &available_tiers);

    if !unknown.is_empty() {
        eprintln!("Unknown technique(s)/tier(s): {}", unknown.join(", "));
        eprintln!("Available techniques:");
        for name in &available_names {
            eprintln!("  {name}");
        }
        eprintln!("Available tiers:");
        for tier in &available_tiers {
            eprintln!("  {}", tier_label(*tier));
        }
        process::exit(2);
    }

    if selectors.is_empty() {
        let puzzle = generator.generate();
        let stats = solve_stats(&solver, &puzzle);
        print_puzzle(&puzzle, &solver, &stats, None, &[]);
        return;
    }

    let selector_labels = selectors
        .iter()
        .map(TechniqueSelector::label)
        .collect::<Vec<_>>();
    let max_tries = args.max_tries.get();

    let (puzzle, stats, score) = (0..max_tries)
        .into_par_iter()
        .map(|_| {
            let puzzle = generator.generate();
            let stats = solve_stats(&solver, &puzzle);
            let score = techniques_score(&solver, &stats, &selectors);
            (puzzle, stats, score)
        })
        .max_by(|a, b| a.2.cmp(&b.2))
        .unwrap();

    print_puzzle(
        &puzzle,
        &solver,
        &stats,
        Some((max_tries, score)),
        &selector_labels,
    );
}

fn build_solver(kind: SolverKind) -> (TechniqueSolver, Vec<&'static str>, BTreeSet<TechniqueTier>) {
    let techniques = match kind {
        SolverKind::All => technique::all_techniques(),
        SolverKind::Fundamental => technique::fundamental_techniques(),
        SolverKind::Basic => technique::basic_techniques(),
    };
    let names = techniques
        .iter()
        .map(|technique| technique.name())
        .collect();
    let tiers = techniques
        .iter()
        .map(|technique| technique.tier())
        .collect();
    (TechniqueSolver::new(techniques), names, tiers)
}

fn parse_technique_selectors(
    inputs: &[String],
    available_names: &[&'static str],
    available_tiers: &BTreeSet<TechniqueTier>,
) -> (Vec<TechniqueSelector>, Vec<String>) {
    let mut selectors = Vec::new();
    let mut unknown = Vec::new();

    for input in inputs {
        if let Some(selector) = TechniqueSelector::parse(input, available_names, available_tiers) {
            selectors.push(selector);
        } else {
            unknown.push(input.clone());
        }
    }

    (selectors, unknown)
}

fn parse_tier(input: &str) -> Option<TechniqueTier> {
    let normalized = input.to_ascii_lowercase().replace('_', "-");
    match normalized.as_str() {
        "fundamental" => Some(TechniqueTier::Fundamental),
        "basic" => Some(TechniqueTier::Basic),
        "intermediate" => Some(TechniqueTier::Intermediate),
        "upper-intermediate" => Some(TechniqueTier::UpperIntermediate),
        "advanced" => Some(TechniqueTier::Advanced),
        "expert" => Some(TechniqueTier::Expert),
        _ => None,
    }
}

fn tier_label(tier: TechniqueTier) -> &'static str {
    match tier {
        TechniqueTier::Fundamental => "fundamental",
        TechniqueTier::Basic => "basic",
        TechniqueTier::Intermediate => "intermediate",
        TechniqueTier::UpperIntermediate => "upper-intermediate",
        TechniqueTier::Advanced => "advanced",
        TechniqueTier::Expert => "expert",
    }
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
    selectors: &[TechniqueSelector],
) -> usize {
    selectors
        .iter()
        .map(|selector| selector.score(solver, stats))
        .sum()
}

fn technique_count(solver: &TechniqueSolver, stats: &TechniqueSolverStats, name: &str) -> usize {
    let Some(i) = solver
        .techniques()
        .iter()
        .position(|technique| technique.name().eq_ignore_ascii_case(name))
    else {
        return 0;
    };
    stats.applications()[i]
}

fn technique_tier_count(
    solver: &TechniqueSolver,
    stats: &TechniqueSolverStats,
    tier: TechniqueTier,
) -> usize {
    solver
        .techniques()
        .iter()
        .enumerate()
        .filter_map(|(i, technique)| (technique.tier() == tier).then_some(stats.applications()[i]))
        .sum()
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
    let mut tier_totals: BTreeMap<TechniqueTier, usize> = BTreeMap::new();
    for (i, count) in stats.applications().iter().enumerate() {
        let technique = &solver.techniques()[i];
        let name = technique.name();
        let tier = technique.tier();
        println!("  {name} ({}): {count}", tier_label(tier));
        *tier_totals.entry(tier).or_insert(0) += count;
    }
    println!("  total: {}", stats.total_steps());
    println!();
    println!("Tier totals:");
    for (tier, total) in tier_totals {
        println!("  {}: {total}", tier_label(tier));
    }
}
