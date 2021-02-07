use crate::game::ExtensiveFormGame;

use crate::game::Player;
use crate::strategy::{BehavioralStrategy, SequenceFormStrategy};
use crate::vector::TreeplexVector;

use crate::treeplex::SequenceId;

/// Struct containing the best response information for blueprints.
/// Computes follower's best response and persistently stores the following intermediate results:
/// (a) follower_bp_br_behavioral
/// (b) follower_bp_br_seqeuence
/// (c) follower_bp_br_seq_values
/// (d) leader_bp_br_seq_values
/// (e) follower-bp_br_behavioral_index
/// TODO (chunkail): Do we really need (a) with the trunk formulation?
///     Unlike the standard BR which we perform on a per-treeplex level, we require that the
///     follower break ties in favour of the leader, so we cannot directly use the best_response
///     method provided by efg. At any rate, this computation is done as a byproduct of computing
///     (b), hence we do not require significantly more computation time.
pub struct BlueprintBr<'a> {
    game: &'a ExtensiveFormGame,
    leader_blueprint: &'a SequenceFormStrategy<'a>,

    // Follower's best 'stackelberg' response to the blueprint,
    // either in behavioral or sequence form.
    follower_sequence: SequenceFormStrategy<'a>,
    follower_behavioral: BehavioralStrategy<'a>,

    // Follower's best response by index, i.e., maps from an infoset index
    // to a sequence index.
    follower_behavioral_index: Vec<SequenceId>,

    // Follower's/Leader's value of each *follower* sequence, given leader's blueprint
    // and stackelberg response from follower. Note that these values are indexed
    // by the *follower* treeplex.
    follower_seq_values: TreeplexVector<'a>,
    leader_seq_values: TreeplexVector<'a>,
}

impl<'a> BlueprintBr<'a> {
    pub fn new(
        game: &'a ExtensiveFormGame,
        leader_blueprint: &'a SequenceFormStrategy,
    ) -> BlueprintBr<'a> {
        let mut grad_follower_payoffs =
            game.gradient_for_payoffs(Player::Player2, Player::Player2, leader_blueprint);
        let mut grad_leader_payoffs =
            game.gradient_for_payoffs(Player::Player2, Player::Player1, leader_blueprint);


        // Now, we compute best responses while breaking ties favoring the leader at each infoset.
        // This is almost identical to the best-response functions within `Treeplex`, except for
        // tiebreaking and the absence of inplace operations.
        let follower_treeplex = game.treeplex(Player::Player2);
        assert_eq!(grad_follower_payoffs.len(), grad_leader_payoffs.len());
        assert_eq!(
            grad_follower_payoffs.len(),
            follower_treeplex.num_sequences()
        );

        let mut behavioral_br = TreeplexVector::from_constant(&follower_treeplex, 0f64);
        let mut behavioral_br_index = Vec::<SequenceId>::new();
        behavioral_br_index.resize(
            follower_treeplex.num_infosets(),
            follower_treeplex.num_sequences(),
        );

        for infoset_id in 0..follower_treeplex.num_infosets() {
            let infoset = follower_treeplex.infosets()[infoset_id];
            let parent_sequence = infoset.parent_sequence;

            // Gets the best sequence only based on follower's payoffs (no tiebreak yet)
            let mut best_follower_value = std::f64::NEG_INFINITY;
            let mut best_follower_index = infoset.start_sequence;
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                if best_follower_value < grad_follower_payoffs[sequence_id] {
                    best_follower_value = grad_follower_payoffs[sequence_id];
                    best_follower_index = sequence_id;
                }
            }

            // Now, go through the sequences again, and for values which are epsilon close
            // to best_follower_value, perform tiebreaking. Tiebreaking is done by referencing
            // the the leader's payoffs.
            let mut best_follower_tiebreak_index = best_follower_index;
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                if abs_diff_eq!(
                    best_follower_value,
                    grad_follower_payoffs[sequence_id],
                    epsilon = 1e-7
                ) {
                    // if relative_eq!(best_follower_value, grad_follower_payoffs[sequence_id]) {
                    if grad_leader_payoffs[sequence_id]
                        > grad_leader_payoffs[best_follower_tiebreak_index]
                    {
                        best_follower_tiebreak_index = sequence_id;
                    }
                }
            }
            behavioral_br_index[infoset_id] = best_follower_tiebreak_index;

            // At this point, the best sequence to be chosen (after tiebreaks) is within
            // best_follower_tiebreak_index. Now, we zero out best responses and set actions
            // behavioral strategies based on the best (tie-breaked) response.
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                behavioral_br[sequence_id] = 0.0;
            }
            behavioral_br[best_follower_tiebreak_index] = 1.0;
            let best_follower_tiebreak_value = grad_follower_payoffs[best_follower_tiebreak_index];
            grad_follower_payoffs[parent_sequence] += best_follower_tiebreak_value;

            // Now, we propagated up the payoffs for the leader as well (albiet up the follower's treeplex).
            // This is correct, since the index to be propagated upwards is based on the best final index, i.e,
            // after tiebreak.
            grad_leader_payoffs[parent_sequence] +=
                grad_leader_payoffs[best_follower_tiebreak_index];
        }

        // Special case of empty sequence
        behavioral_br[follower_treeplex.empty_sequence_id()] = 1f64;
        let behavioral_br = BehavioralStrategy::from_treeplex_vector(behavioral_br);

        BlueprintBr {
            game,
            leader_blueprint,
            follower_seq_values: grad_follower_payoffs,
            follower_sequence: SequenceFormStrategy::from_behavioral_strategy(
                behavioral_br.clone(),
            ),
            follower_behavioral: behavioral_br,
            leader_seq_values: grad_leader_payoffs,
            follower_behavioral_index: behavioral_br_index,
        }

    }

    pub fn leader_blueprint(&self) -> &SequenceFormStrategy<'a> {
        self.leader_blueprint
    }

    pub fn follower_seq_values(&self) -> &TreeplexVector<'a> {
        &self.follower_seq_values
    }

    pub fn follower_seq_value(&self, seq_id: SequenceId) -> f64 {
        self.follower_seq_values[seq_id]
    }

    pub fn leader_seq_values(&self) -> &TreeplexVector<'a> {
        &self.leader_seq_values
    }

    pub fn leader_seq_value(&self, seq_id: SequenceId) -> f64 {
        self.leader_seq_values[seq_id]
    }

    pub fn follower_behavioral_index(&self, infoset_id: usize) -> usize {
        self.follower_behavioral_index[infoset_id]
    }

    pub fn follower_behavioral(&self, seq_id: SequenceId) -> f64 {
        self.follower_behavioral.inner()[seq_id]
    }

    pub fn follower_behavioral_strategy(&self) -> &BehavioralStrategy {
        &self.follower_behavioral
    }

    pub fn follower_sequence(&self) -> &SequenceFormStrategy<'a> {
        &self.follower_sequence
    }

    pub fn follower_infoset_value(&self, infoset_id: usize) -> f64 {
        self.follower_seq_value(self.follower_behavioral_index(infoset_id))
    }
}