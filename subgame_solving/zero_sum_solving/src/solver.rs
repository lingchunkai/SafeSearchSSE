use efg_lite::game::ExtensiveFormGame;
use crate::zero_sum_solution::ZeroSumSolution;

pub trait Solver<'a> {
    fn new(game: &'a ExtensiveFormGame, solver_config: &'a SolverConfig) -> Self;
    fn solve(&self);
    fn get_solution(&self) -> ZeroSumSolution<'a>;
}

pub struct SolverConfig {
    pub time_limit: f64,
}