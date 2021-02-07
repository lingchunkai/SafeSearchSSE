use crate::mip_solution::MIPSolution;
use efg_lite::sse::BoundedProblem;

pub trait Solver<'a> {
    fn new(problem: &'a BoundedProblem, solver_config: &SolverConfig) -> Self;
    fn solve(&self);
    fn get_solution(&self) -> MIPSolution<'a>;
}

pub struct SolverConfig {
    pub time_limit: f64,
}