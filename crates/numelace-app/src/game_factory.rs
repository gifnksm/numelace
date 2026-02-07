use numelace_generator::{GeneratedPuzzle, PuzzleGenerator};
use numelace_solver::TechniqueSolver;

pub(crate) fn generate_random_puzzle() -> GeneratedPuzzle {
    let technique_solver = TechniqueSolver::with_all_techniques();
    PuzzleGenerator::new(&technique_solver).generate()
}
