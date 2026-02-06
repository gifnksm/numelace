use numelace_game::Game;
use numelace_generator::PuzzleGenerator;
use numelace_solver::TechniqueSolver;

use crate::async_work::new_game_dto::NewGameDto;

#[must_use]
pub fn generate_random_game() -> Game {
    let technique_solver = TechniqueSolver::with_all_techniques();
    let puzzle = PuzzleGenerator::new(&technique_solver).generate();
    Game::new(puzzle)
}

#[must_use]
pub fn generate_new_game_dto() -> NewGameDto {
    let technique_solver = TechniqueSolver::with_all_techniques();
    let puzzle = PuzzleGenerator::new(&technique_solver).generate();
    NewGameDto {
        problem: puzzle.problem.to_string(),
        solution: puzzle.solution.to_string(),
    }
}
