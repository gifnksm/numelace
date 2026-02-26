use std::iter;

use numelace_core::DigitGrid;
use numelace_generator::{GeneratedPuzzle, PuzzleGenerator, PuzzleSeed};
use numelace_solver::{TechniqueGrid, TechniqueSolver, TechniqueTier, technique};
use serde::{Deserialize, Serialize};

use crate::worker::tasks::GeneratePuzzleRequestDto;

/// DTO for communicating newly generated Sudoku puzzles over worker boundaries.
///
/// Uses compact 81-char string formats ('.' for empty, '1'..'9' for digits).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GeneratedPuzzleDto {
    pub(crate) seed: String,
    pub(crate) problem: String,
    pub(crate) solution: String,
}

impl From<GeneratedPuzzle> for GeneratedPuzzleDto {
    fn from(puzzle: GeneratedPuzzle) -> Self {
        Self {
            seed: puzzle.seed.to_string(),
            problem: puzzle.problem.to_string(),
            solution: puzzle.solution.to_string(),
        }
    }
}

impl TryFrom<GeneratedPuzzleDto> for GeneratedPuzzle {
    type Error = String;

    fn try_from(value: GeneratedPuzzleDto) -> Result<Self, Self::Error> {
        let seed = value.seed.parse()?;
        let problem = value
            .problem
            .parse::<DigitGrid>()
            .map_err(|e| e.to_string())?;
        let solution = value
            .solution
            .parse::<DigitGrid>()
            .map_err(|e| e.to_string())?;
        Ok(GeneratedPuzzle {
            seed,
            problem,
            solution,
        })
    }
}

pub(crate) fn generate_puzzle(request: &GeneratePuzzleRequestDto) -> GeneratedPuzzleDto {
    let mut techniques = vec![];
    for id in &request.techniques {
        if let Some(technique) = technique::find_technique_by_id(id) {
            techniques.push(technique);
        }
    }
    let technique_solver = TechniqueSolver::new(techniques);
    let puzzle = if request.seed.is_empty() {
        generate_random_puzzle(&technique_solver, request.max_attempts)
    } else {
        generate_seeded_puzzle(&request.seed, &technique_solver)
    };
    puzzle.into()
}

fn generate_random_puzzle(
    technique_solver: &TechniqueSolver,
    max_attempts: usize,
) -> GeneratedPuzzle {
    let max_tier = technique_solver
        .techniques()
        .iter()
        .map(|t| t.tier())
        .max()
        .unwrap();
    let mut best: Option<(TechniqueTier, usize, GeneratedPuzzle)> = None;
    for _ in 0..max_attempts.max(1) {
        let puzzle = PuzzleGenerator::new(technique_solver).generate();
        let Ok((true, stats)) =
            technique_solver.solve_with_step(&mut TechniqueGrid::from_digit_grid(&puzzle.problem))
        else {
            continue;
        };
        let (tier, app) = iter::zip(
            technique_solver.techniques().iter().map(|t| t.tier()),
            stats.applications().iter().copied(),
        )
        .rfind(|(_tech, app)| *app > 0)
        .unwrap_or((TechniqueTier::Fundamental, 0));
        if tier >= max_tier {
            return puzzle;
        }
        if best
            .as_ref()
            .is_none_or(|(best_tier, best_app, _puzzle)| (tier, app) > (*best_tier, *best_app))
        {
            best = Some((tier, app, puzzle));
        }
    }
    best.unwrap().2
}

fn generate_seeded_puzzle(seed: &str, technique_solver: &TechniqueSolver) -> GeneratedPuzzle {
    let seed = parse_seed(seed);
    PuzzleGenerator::new(technique_solver).generate_with_seed(seed)
}

fn parse_seed(seed: &str) -> PuzzleSeed {
    if let Ok(seed) = seed.parse() {
        return seed;
    }
    PuzzleSeed::from_arbitrary_bytes(seed.as_bytes())
}
