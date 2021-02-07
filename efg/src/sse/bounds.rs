use crate::game::{
    ExtensiveFormGame, Infoset, PayoffMatrix, PayoffMatrixEntry, Player, SubgameOrFree,
};
use crate::sse::{BlueprintBr, TreeplexMapper, ValueBound};
use crate::treeplex::{SequenceId, Treeplex, TreeplexTools};

/// This struct provides methods to compute the `safety-bounds' for the follower
/// information sets with respect to some game and a given blueprint. This is done by
/// the following tree-traversal steps.
///
/// 1) Expand the follower's treeplex top down while generating bounds for each infoset
/// and sequences. We terminate the expansion whenever we bump into a subgame. These infosets
/// which terminate the tree traversal are known as `heads'.
/// 2) For each head, we store the constraint that needs to be satisfied at these head nodes
/// in order to ensure that the follower best-response does not change when compared
/// with the blueprint, even if subgame-solving is applied to all subgames.
///
/// The chief complexity here is the way in which these bounds are generated, similar
/// to the `gift-splitting' procedure originally used in Libratus.

pub struct BoundsGenerator<'a> {
    game: &'a ExtensiveFormGame,
    follower_treeplex_tools: &'a TreeplexTools,
    leader_treeplex_tools: &'a TreeplexTools,
    blueprint_br: &'a BlueprintBr<'a>,
    splitting_ratio: f64,
    gift_factor: f64,
}

impl<'a> BoundsGenerator<'a> {
    pub fn new(
        game: &'a ExtensiveFormGame,
        follower_treeplex_tools: &'a TreeplexTools,
        leader_treeplex_tools: &'a TreeplexTools,
        blueprint_br: &'a BlueprintBr<'a>,
        splitting_ratio: f64,
        gift_factor: f64,
    ) -> BoundsGenerator<'a> {
        assert!(splitting_ratio >= 0.0 && splitting_ratio <= 1.0);
        BoundsGenerator {
            game,
            follower_treeplex_tools,
            leader_treeplex_tools,
            blueprint_br,
            splitting_ratio,
            gift_factor,
        }
    }

    pub fn follower_bounds(&self) -> Vec<ValueBound> {
        let treeplex = self.game.treeplex(Player::Player2);
        let mut follower_payoff_bounds =
            std::vec::from_elem::<ValueBound>(ValueBound::None, treeplex.num_infosets());

        self.expand_seq_trunk(
            treeplex.empty_sequence_id(),
            std::f64::NEG_INFINITY,
            &mut follower_payoff_bounds,
        );

        follower_payoff_bounds
    }

    /// Expands all infosets under a sequence within a trunk.
    /// Gifts for now are split uniformly among children infosets:
    /// TODO (chunkail) split between subgames! This may possibly be done by
    /// bottom up traversal of the treeplex and doing some form of `forward-backward`
    /// algorithm.
    fn expand_seq_trunk(
        &self,
        sequence_id: SequenceId,
        lower_bound: f64,
        follower_payoff_bounds: &mut Vec<ValueBound>,
    ) {
        // TODO (chunkail) handle case where this a leaf or subgame.
        let bp_br_value = self.blueprint_br.follower_seq_value(sequence_id);
        assert!(
            approx_ge(bp_br_value, lower_bound),
            "bp br value {:?}, lower bound {:?}",
            bp_br_value,
            lower_bound
        );

        let gift_value = f64::max(0f64, bp_br_value - lower_bound) * self.gift_factor;
        let gift_spread = gift_value
            / (self
                .follower_treeplex_tools
                .seq_to_infoset_range(sequence_id)
                .len() as f64);

        for next_infoset_id in self
            .follower_treeplex_tools
            .seq_to_infoset_range(sequence_id)
        {
            let next_infoset_value = self.blueprint_br.follower_infoset_value(next_infoset_id);
            self.expand_infoset_trunk(
                next_infoset_id,
                next_infoset_value - gift_spread,
                follower_payoff_bounds,
            );
        }
    }

    /// Expands all infosets under a sequence not within a trunk.
    fn expand_seq_nontrunk(
        &self,
        sequence_id: SequenceId,
        upper_bound: f64,
        follower_payoff_bounds: &mut Vec<ValueBound>,
    ) {
        // TODO (chunkail) handle case where this is a leaf or subgame.
        let bp_br_value = self.blueprint_br.follower_seq_value(sequence_id);
        assert!(
            approx_ge(upper_bound, bp_br_value),
            "upper bound {:?}, bp_br_value {:?}",
            upper_bound,
            bp_br_value
        );

        let gift_value = f64::min(0f64, upper_bound - bp_br_value) * self.gift_factor;
        let gift_spread = gift_value
            / (self
                .follower_treeplex_tools
                .seq_to_infoset_range(sequence_id)
                .len() as f64);

        for next_infoset_id in self
            .follower_treeplex_tools
            .seq_to_infoset_range(sequence_id)
        {
            let next_infoset_value = self.blueprint_br.follower_infoset_value(next_infoset_id);
            self.expand_infoset_nontrunk(
                next_infoset_id,
                next_infoset_value + gift_spread,
                follower_payoff_bounds,
            );
        }
    }

    fn expand_infoset_trunk(
        &self,
        infoset_id: usize,
        lower_bound: f64,
        follower_payoff_bounds: &mut Vec<ValueBound>,
    ) {
        match self.game.subgame(Player::Player2, infoset_id) {
            SubgameOrFree::Subgame(x) => {
                follower_payoff_bounds[infoset_id] = ValueBound::LowerBound(lower_bound);
            }
            SubgameOrFree::Free => {
                // "Local" threshold by virtue of the blueprint values.
                let threshold = self.threshold_from_child_seqs(infoset_id);

                // Get tighter of the threshold compared to the propagated bounds.
                let threshold = f64::max(threshold, lower_bound);

                // Expand each child sequence based on whether they are part of the trunk
                // or otherwise.
                let treeplex = self.game.treeplex(Player::Player2);
                let infoset = treeplex.infosets()[infoset_id];
                for seq_id in (infoset.start_sequence..=infoset.end_sequence).into_iter() {
                    if self.blueprint_br.follower_behavioral_index(infoset_id) == seq_id {
                        self.expand_seq_trunk(seq_id, threshold, follower_payoff_bounds);
                    } else {
                        self.expand_seq_nontrunk(seq_id, threshold, follower_payoff_bounds);
                    }
                }
            }
        }
    }

    fn expand_infoset_nontrunk(
        &self,
        infoset_id: usize,
        upper_bound: f64,
        follower_payoff_bounds: &mut Vec<ValueBound>,
    ) {
        match self.game.subgame(Player::Player2, infoset_id) {
            SubgameOrFree::Subgame(x) => {
                follower_payoff_bounds[infoset_id] = ValueBound::UpperBound(upper_bound);
            }
            SubgameOrFree::Free => {
                let treeplex = self.game.treeplex(Player::Player2);
                let infoset = treeplex.infosets()[infoset_id];
                for seq_id in (infoset.start_sequence..=infoset.end_sequence).into_iter() {
                    self.expand_seq_nontrunk(seq_id, upper_bound, follower_payoff_bounds);
                }
            }
        }
    }

    fn threshold_from_child_seqs(&self, infoset_id: usize) -> f64 {
        // Set an upper bound on the lower bound based off the top 2 sequences at this infoset.
        let treeplex = self.game.treeplex(Player::Player2);
        let infoset = treeplex.infosets()[infoset_id];

        // Get maximum sequence value and its index.
        let (best_index, best_value) = (infoset.start_sequence..=infoset.end_sequence)
            .into_iter()
            .fold(
                (treeplex.num_sequences(), std::f64::NEG_INFINITY),
                |s, x| {
                    let value = self.blueprint_br.follower_seq_value(x);
                    if value > s.1 {
                        (x, value)
                    } else {
                        s
                    }
                },
            );

        // Get second maximum sequence value. If there is only one action, then this is -infinity
        let second_best_value = (infoset.start_sequence..=infoset.end_sequence)
            .into_iter()
            .fold(std::f64::NEG_INFINITY, |s, x| {
                if x == best_index {
                    s
                } else {
                    let value = self.blueprint_br.follower_seq_value(x);
                    if value > s {
                        value
                    } else {
                        s
                    }
                }
            });

        // Our current implementation takes the average of best and second best action, though need
        // not necessarily be the case.
        second_best_value + (best_value - second_best_value) * self.splitting_ratio
    }
}

/// Checks if a >= b
fn approx_ge(a: f64, b: f64) -> bool {
    a >= b || relative_eq!(a, b) || ulps_eq!(a, b)
}