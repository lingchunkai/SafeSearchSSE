
use efg_lite::vector::TreeplexVector;
use efg_lite::strategy::SequenceFormStrategy;

pub struct ZeroSumSolution<'a> {
    pub strategy_pl1: SequenceFormStrategy<'a>,
    pub game_value: f64,
}

impl<'a> ZeroSumSolution<'a> {
    pub fn new(
        strategy_pl1: SequenceFormStrategy<'a>,
        game_value: f64,
    ) -> ZeroSumSolution<'a> {
        ZeroSumSolution {
            strategy_pl1,
            game_value,
        }
    }
}
