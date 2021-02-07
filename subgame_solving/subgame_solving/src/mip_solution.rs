use efg_lite::vector::TreeplexVector;
use efg_lite::strategy::SequenceFormStrategy;

pub struct MIPSolution<'a> {
    pub leader_strategy: SequenceFormStrategy<'a>,
    pub follower_strategy: SequenceFormStrategy<'a>,
    pub leaf_probabilities: Vec<f64>,
    pub objective_value: f64,
    pub follower_slack: TreeplexVector<'a>,
    pub follower_value: Vec<f64>,
}

impl<'a> MIPSolution<'a> {
    pub fn new(
        leader_strategy: SequenceFormStrategy<'a>,
        follower_strategy: SequenceFormStrategy<'a>,
        leaf_probabilities: Vec<f64>,
        objective_value: f64,
        follower_slack: TreeplexVector<'a>,
        follower_value: Vec<f64>,
    ) -> MIPSolution<'a> {
        let solution = MIPSolution {
            leader_strategy,
            follower_strategy,
            leaf_probabilities,
            objective_value,
            follower_slack,
            follower_value,
        };
        // solution.verify();
        solution
    }

    /// Verify Big M constraints were satisfied
    pub fn verify(&self) {
        let treeplex = self.follower_strategy.inner().treeplex();
        let num_sequences = treeplex.num_sequences();
        for sequence_id in 0..num_sequences {
            assert!(abs_diff_eq!(self.follower_strategy.inner()[sequence_id], 1.0) || 
                    abs_diff_eq!(self.follower_strategy.inner()[sequence_id], 0.0));
            if abs_diff_eq!(self.follower_strategy.inner()[sequence_id], 1.0) {
                assert_abs_diff_eq!(self.follower_slack[sequence_id], 0.0, epsilon=1e-6);
            }           
        }
    }
}

