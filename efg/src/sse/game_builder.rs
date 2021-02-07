use crate::game::{
    ExtensiveFormGame, Infoset, PayoffMatrix, PayoffMatrixEntry, Player, SubgameOrFree,
};
use crate::sse::{
    BlueprintBr, BoundedProblem, BoundsGenerator, GameMapper, TreeplexBuilder, TreeplexMapper,
    ValueBound,
};
use crate::strategy::{BehavioralStrategy, SequenceFormStrategy};
use crate::vector::TreeplexVector;
use crate::treeplex::{SequenceId, Treeplex, TreeplexTools};
use std::rc::Rc;

/* TODO (chunkail): Major overhaul to move away from recursive functions
into iterative ones. */

/// Performs top down traversal of the follower treeplex and does one of the following for
/// each infoset/sequence:
/// (I) If infoset is not inside any subgame, i.e., free, then
///     (a) trunk sequences: create an lower bound on the value of the sequence.
///     (b) non-trunk sequences: create an upper bound on the value of the sequence.
/// (II) If infoset is inside a subgame not equal to subgame_index, then
///     (a) replace it with the payoffs of the blueprint.
/// (III) If infoset is inside the subgame indexed by subgame_index, then
///     (a) keep the subgame as it is.
pub struct GameBuilder<'a> {
    game: &'a ExtensiveFormGame,
    follower_treeplex_tools: &'a TreeplexTools,
    leader_treeplex_tools: &'a TreeplexTools,
    blueprint_br: &'a BlueprintBr<'a>,

    // Bounds on value for each of the follower infosets.
    follower_payoff_bounds: Vec<ValueBound>,

    // Possible head infosets for each subgame, for each player.
    follower_heads_per_subgame: Vec<Vec<usize>>,
    leader_heads_per_subgame: Vec<Vec<usize>>,

    // For each leader sequence, store head infoset within subgame, or None otherwise.
    leader_seq_to_head: Vec<Option<usize>>,
    follower_seq_to_head: Vec<Option<usize>>,
}

impl<'a> GameBuilder<'a> {
    pub fn new(
        game: &'a ExtensiveFormGame,
        follower_treeplex_tools: &'a TreeplexTools,
        leader_treeplex_tools: &'a TreeplexTools,
        blueprint_br: &'a BlueprintBr<'a>,
        splitting_ratio: f64,
        gift_factor: f64,
    ) -> GameBuilder<'a> {
        let (follower_heads_per_subgame, follower_seq_to_head) =
            Self::preprocess_treeplex_subgames(game, Player::Player2, follower_treeplex_tools);
        let (leader_heads_per_subgame, leader_seq_to_head) =
            Self::preprocess_treeplex_subgames(game, Player::Player1, leader_treeplex_tools);

        /*
        println!("{:?}", leader_treeplex_tools);
        println!("Leader seq to head {:?}", leader_seq_to_head);
        println!("Leader head per subgame {:?}", leader_heads_per_subgame);
        println!("Follower seq to head {:?}", follower_seq_to_head);
        */

        let bounds_generator = BoundsGenerator::new(
            game,
            follower_treeplex_tools,
            leader_treeplex_tools,
            blueprint_br,
            splitting_ratio,
            gift_factor,
        );
        let follower_payoff_bounds = bounds_generator.follower_bounds();

        GameBuilder {
            game,
            follower_treeplex_tools,
            leader_treeplex_tools,
            blueprint_br,
            follower_payoff_bounds,
            follower_heads_per_subgame,
            leader_heads_per_subgame,
            follower_seq_to_head,
            leader_seq_to_head,
        }
    }

    /// Construct skinny game for a given subgame. This involves:
    /// (I) Constructing the 2 new skinny treeplexes.
    /// (II) Mapping the SequenceIDs (and possibly infoset ids) from the new treeplexes to old treeplexes.
    /// (III) New payoff matrix, which
    ///       (a) Uses the new (smaller) number of sequence ids,
    ///       (b) Modulates leaf chance factors by follower sequences before subgames.
    /// (IV) For each follower infoset that is the root of the given subgame, give bounds on the maximum/minimum
    /// payoff for that infoset.
    /// TODO (chunkail) Is there any possibility of having different resolving schemes for different
    /// information sets?
    pub fn bounded_problem(&self, subgame_id: usize) -> (BoundedProblem, 
                                                         GameMapper, 
                                                         Vec<f64>, // TODO change to returning strategy.
                                                         Vec<f64>) {
        assert!(subgame_id < self.game.num_subgames(), "Invalid subgame id");

        // First construct SkinnyGame.
        let (treeplex_follower, mapper_follower) =
            self.skinny_treeplex(subgame_id, Player::Player2);
        let (treeplex_leader, mapper_leader) = self.skinny_treeplex(subgame_id, Player::Player1);
        let skinny_payoff_matrix =
            self.skinny_payoff_matrix(subgame_id, &mapper_leader, &mapper_follower);
        let skinny_game = ExtensiveFormGame::new(
            Rc::<Treeplex>::new(treeplex_leader),
            Rc::<Treeplex>::new(treeplex_follower),
            skinny_payoff_matrix,
            vec![],
            vec![],
        );

        // Compute total probability mass entering subgame, assuming *both* players
        // play the blueprints strategy. Include chance nodes.
        // In the original MILP formulation, this is equal
        // to 1, but in the subgame version, this is not necessarily the case since
        // mass has leaked out (e.g., via chance) to sections of the tree not within the
        // subgame.
        let input_mass = self
            .game
            .payoff_matrix()
            .entries
            .iter()
            .filter(|payoff_entry| {
                let follower_head = self.follower_seq_to_head[payoff_entry.seq_pl2];
                let leader_head = self.leader_seq_to_head[payoff_entry.seq_pl1];

                /*
                follower_head.is_some()
                    && leader_head.is_some()
                    && SubgameOrFree::Subgame(subgame_id)
                        == self.game.subgame(Player::Player2, follower_head.unwrap())
                    && SubgameOrFree::Subgame(subgame_id)
                        == self.game.subgame(Player::Player1, leader_head.unwrap())
                */

                /* TODO, special case 
                */
                (follower_head.is_some() && 
                SubgameOrFree::Subgame(subgame_id) == self.game.subgame(Player::Player2, follower_head.unwrap())) ||
                (leader_head.is_some() &&
                SubgameOrFree::Subgame(subgame_id) == self.game.subgame(Player::Player1, leader_head.unwrap()))
            }) // TODO (chunkail): do this properly.
            .fold(0.0, |accum, payoff_entry| {
                    accum
                        + self.blueprint_br.follower_sequence().inner()[payoff_entry.seq_pl2]
                            * self.blueprint_br.leader_blueprint().inner()[payoff_entry.seq_pl1]
                            * payoff_entry.chance_factor
            });

        // Extract bounds of follower coressponding only to the new infoset_ids.
        let bounds = self.follower_heads_per_subgame[subgame_id]
            .iter()
            .map(|x| {
                (
                    mapper_follower.infoset_to_skinny_infoset(*x),
                    self.follower_payoff_bounds[*x],
                )
            })
            .collect::<Vec<(usize, ValueBound)>>();

        // Indices of leaves (in skinny treeplex) which are led to
        // by follower "pre-subgame" blueprint strategy.
        let leaves_within_trunk = (0..skinny_game.payoff_matrix().entries.len())
            .filter(|&x| {
                let original_seq_id = mapper_follower
                    .skinny_seq_to_seq(skinny_game.payoff_matrix().entries[x].seq_pl2);
                // let head_infoset_id = self.leader_seq_to_head[original_seq_id].unwrap();
                let head_infoset_id = self.follower_seq_to_head[original_seq_id].unwrap();
                let head_infoset = self.game.treeplex(Player::Player2).infosets()[head_infoset_id];
                let head_sequence_id = head_infoset.parent_sequence;
                self.blueprint_br.follower_sequence().inner()[head_sequence_id] > 0.5
            })
            .collect::<Vec<usize>>();

        let bounded_problem = BoundedProblem::new(skinny_game, input_mass, bounds, leaves_within_trunk);
        let game_mapper = GameMapper::new(mapper_leader, mapper_follower);

        // TODO(chunkail): Move this verification step to a separate function.
        // 
        // Verify if blueprint satisfies bounds under the mapper.
        // Extract skinny leader blueprint.
        let bp_leader_seq_form = self.blueprint_br.leader_blueprint().clone();
        let bp_leader_beh_form = BehavioralStrategy::from_sequence_form_strategy(bp_leader_seq_form);
        let skinny_treeplex = bounded_problem.game.treeplex(Player::Player1);
        let mut skinny_bp_leader = TreeplexVector::from_constant(skinny_treeplex, 1f64);
        for seq_id in 0..skinny_treeplex.num_sequences()-1 {
            let full_seq = game_mapper.mapper_leader.skinny_seq_to_seq(seq_id).clone();
            skinny_bp_leader[seq_id] = bp_leader_beh_form.inner()[full_seq];
        }
        let skinny_bp_leader = BehavioralStrategy::from_treeplex_vector(skinny_bp_leader);
        let skinny_bp_leader = SequenceFormStrategy::from_behavioral_strategy(skinny_bp_leader);

        // Extract skinny follower blueprint.
        let bp_follower_seq_form = self.blueprint_br.follower_sequence().clone();
        let bp_follower_beh_form = BehavioralStrategy::from_sequence_form_strategy(bp_follower_seq_form);
        let skinny_treeplex = bounded_problem.game.treeplex(Player::Player2);
        let mut skinny_bp_follower = TreeplexVector::from_constant(skinny_treeplex, 1f64);
        for seq_id in 0..skinny_treeplex.num_sequences()-1 {
            let full_seq = game_mapper.mapper_follower.skinny_seq_to_seq(seq_id).clone();
            skinny_bp_follower[seq_id] = bp_follower_beh_form.inner()[full_seq];
        }
        let skinny_bp_follower = BehavioralStrategy::from_treeplex_vector(skinny_bp_follower);
        let skinny_bp_follower = SequenceFormStrategy::from_behavioral_strategy(skinny_bp_follower);

        // Get gradient.
        let gradient_follower = bounded_problem.game.gradient(Player::Player2, &skinny_bp_leader);
        
        // Compute values upwards.
        let treeplex = skinny_treeplex;
        let mut sequence_values = gradient_follower.clone();
        let mut infoset_values = std::vec::from_elem::<f64>(-std::f64::INFINITY, treeplex.num_infosets());
        let mut touched = std::vec::from_elem::<bool>(false, treeplex.num_sequences());
        for infoset_id in 0..skinny_treeplex.num_infosets() {
            let infoset = bounded_problem.game.treeplex(Player::Player2).infosets()[infoset_id];
            for seq_id in infoset.start_sequence..=infoset.end_sequence {
                infoset_values[infoset_id] = infoset_values[infoset_id].max(sequence_values[seq_id]);
            }
            sequence_values[infoset.parent_sequence] += infoset_values[infoset_id];
        }

        /*
        println!("{:?}", infoset_values);
        println!("{:?}", bounded_problem.bounds);

        for k in bounded_problem.bounds.clone() {
            let z = k.0;
            println!("{:?}---{:?}, {:?}", z, infoset_values[z], k.1);
        }
        */

        for k in bounded_problem.bounds.clone() {
            let bound_direction = k.1;
            let infoset_value = infoset_values[k.0];
            match bound_direction {
                ValueBound::LowerBound(x) => {
                    assert!(infoset_value - x >= -1e-7, "{:?}", infoset_value -x );
                }
                ValueBound::UpperBound(x) => {
                    assert!(x - infoset_value >= -1e-7, "{:?}", x - infoset_value);
                }
                ValueBound::None => {}
            }
        }

        let feasible_leader = self.get_mapped_blueprint_solution(
                                &bounded_problem.game.treeplex(Player::Player1),
                                &game_mapper,
                                Player::Player1,
                                );
        let feasible_follower = self.get_mapped_blueprint_solution(
                                &bounded_problem.game.treeplex(Player::Player2),
                                &game_mapper,
                                Player::Player2,
                                );


        (bounded_problem, game_mapper, feasible_leader, feasible_follower)
    }

    fn get_mapped_blueprint_solution(&self, 
                                     treeplex: &Treeplex,
                                     game_mapper: &GameMapper,
                                     player: Player) ->  Vec<f64>{
        // Now we construct a feasible strategy in the skinny game using the BP strategy.
        // let treeplex = bounded_problem.game.treeplex(player).clone();// Whatever, borrow checker.
        let mut v = std::vec::from_elem(1f64, treeplex.num_sequences());
        let beh_strategy = match(player) {
            Player::Player1 => {
                BehavioralStrategy::from_sequence_form_strategy(
                    self.blueprint_br.leader_blueprint().clone()
                )
            },
            Player::Player2 => {
                self.blueprint_br.follower_behavioral_strategy().clone()
            }
        };

        for seq_id in 0..treeplex.num_sequences() {
            if seq_id == treeplex.empty_sequence_id() { 
                // v[seq_id] = 1f64;
                continue;
            }
            let original_seq_id = match(player) { 
                Player::Player1 => game_mapper.mapper_leader.skinny_seq_to_seq(seq_id),
                Player::Player2 => game_mapper.mapper_follower.skinny_seq_to_seq(seq_id),
            };

            v[seq_id] = beh_strategy.inner()[original_seq_id];
        }
        // println!("{:?} {:?}", v, treeplex.num_infosets());
        let q = TreeplexVector::from_vec(&treeplex, v.clone());
        let w = BehavioralStrategy::from_treeplex_vector(q);
        let r = SequenceFormStrategy::from_behavioral_strategy(w);
        r.inner().entries.clone()
    }

    /// Construst skinny treeplex for players and their mappers.
    fn skinny_treeplex(&self, subgame_id: usize, player: Player) -> (Treeplex, TreeplexMapper) {
        let treeplex = self.game.treeplex(player);
        let is_relevant_infoset = self.relevant_infosets(player, subgame_id);
        let treeplex_mapper = TreeplexMapper::new(treeplex, &is_relevant_infoset);
        let treeplex_builder = TreeplexBuilder::new(treeplex);
        let skinny_treeplex = treeplex_builder.treeplex_from_mapper(&treeplex_mapper);

        (skinny_treeplex, treeplex_mapper)
    }

    // TODO (chunkail) splitting payoffs into subgames may be preprocessed and computed
    // for all subgames at once.
    fn skinny_payoff_matrix(
        &self,
        subgame_id: usize,
        mapper_leader: &TreeplexMapper,
        mapper_follower: &TreeplexMapper,
    ) -> PayoffMatrix {
        let mut payoff_matrix = Vec::<PayoffMatrixEntry>::new();

        // Get payoffs which are at descendents of subgame_id.
        for payoff_entry in self
            .game
            .payoff_matrix()
            .entries
            .iter()
            .filter(|&payoff_entry| {
                // By definition of a subgame, if a payoff-sequence for a payoff entry lies
                // within some subgame-treeplex for one player, then the
                // payoff-sequence for the other player *must* also lie within the same subgame-treeplex
                // except for the fact that the treeplex belongs to another player.
                // OR, the 
                let is_in_subgame = mapper_leader.is_sequence_mapped(payoff_entry.seq_pl1) ||
                                    mapper_follower.is_sequence_mapped(payoff_entry.seq_pl2);
                is_in_subgame
            })
        {
            // Check if payoff belongs to the same subgame (or Free), but checking
            // if *both* sequences for this leaf lies within the same subgame.
            /*
            assert_eq!(
                {
                    let parent_infoset = self
                        .leader_treeplex_tools
                        .parent_infoset_of_seq(payoff_entry.seq_pl1);
                    match parent_infoset {
                        None => SubgameOrFree::Free,
                        Some(x) => self.game.subgame(Player::Player1, x),
                    }
                },
                {
                    let parent_infoset = self
                        .follower_treeplex_tools
                        .parent_infoset_of_seq(payoff_entry.seq_pl2);
                    match parent_infoset {
                        None => SubgameOrFree::Free,
                        Some(x) => self.game.subgame(Player::Player2, x),
                    }
                }
            );
            */

            // If both sequences are within the same subgame, then we are done
            if mapper_leader.is_sequence_mapped(payoff_entry.seq_pl1) && mapper_follower.is_sequence_mapped(payoff_entry.seq_pl2) {
                let skinny_seq_pl1 = mapper_leader.seq_to_skinny_seq(payoff_entry.seq_pl1);
                let skinny_seq_pl2 = mapper_follower.seq_to_skinny_seq(payoff_entry.seq_pl2);

                // We also need to modulate the chance factor by the probabilities that the
                // *leader* took in his moves prior to entering the subgame.
                // We should *not* do this for the follower since the bounds are required
                // to be held *before* modulation (which will be binary) for the follower.
                let leader_chance_factor = {
                    // println!("{:?}", payoff_entry.seq_pl1);
                    let head_infoset_id = self.leader_seq_to_head[payoff_entry.seq_pl1].unwrap();
                    let head_infoset = self.game.treeplex(Player::Player1).infosets()[head_infoset_id];
                    let head_sequence_id = head_infoset.parent_sequence;
                    self.blueprint_br.leader_blueprint().inner()[head_sequence_id]
                };
                payoff_matrix.push(PayoffMatrixEntry::new(
                    skinny_seq_pl1,
                    skinny_seq_pl2,
                    payoff_entry.chance_factor * leader_chance_factor,
                    payoff_entry.payoff_pl1,
                    payoff_entry.payoff_pl2,
                )); 
            } else if mapper_leader.is_sequence_mapped(payoff_entry.seq_pl1) {
                panic!("Not implemetned yet");
                println!("TRICKY1");
                // Tricky
                let skinny_seq_pl1 = mapper_leader.seq_to_skinny_seq(payoff_entry.seq_pl1);
                let skinny_seq_pl2 = mapper_follower.skinny_empty_seq();

                let leader_chance_factor = {
                    let head_infoset_id = self.leader_seq_to_head[payoff_entry.seq_pl1].unwrap();
                    let head_infoset = self.game.treeplex(Player::Player1).infosets()[head_infoset_id];
                    let head_sequence_id = head_infoset.parent_sequence;
                    self.blueprint_br.leader_blueprint().inner()[head_sequence_id]
                };

                // Follower chance factor will be binary.
                // let follower_prob = self.blueprint_br.follower_sequence()[payoff_entry.seq_pl2];

                // if follower_prob > 0.5 { // anything > 0 is technically good
                payoff_matrix.push(PayoffMatrixEntry::new(
                    skinny_seq_pl1,
                    skinny_seq_pl2,
                    payoff_entry.chance_factor * leader_chance_factor,
                    payoff_entry.payoff_pl1,
                    payoff_entry.payoff_pl2,
                ));
                //}
            } else if mapper_follower.is_sequence_mapped(payoff_entry.seq_pl2) {
                // Follower sequence is inside, the subgame, but leader sequence isn't.

                // panic!("Not implemetned yet");

                println!("TRICKY2");
                // Tricky.
                let skinny_seq_pl1 = mapper_leader.skinny_empty_seq();
                let skinny_seq_pl2 = mapper_follower.seq_to_skinny_seq(payoff_entry.seq_pl2);
                // println!("SKINNY_SEQ {:?} {:?}", skinny_seq_pl1, skinny_seq_pl2);

                let leader_chance_factor = {
                    /*
                    let head_infoset_id = self.leader_seq_to_head[payoff_entry.seq_pl1].unwrap();
                    let head_infoset = self.game.treeplex(Player::Player1).infosets()[head_infoset_id];
                    let head_sequence_id = head_infoset.parent_sequence;
                    self.blueprint_br.leader_blueprint().inner()[head_sequence_id]
                    */
                    self.blueprint_br.leader_blueprint().inner()[payoff_entry.seq_pl1]
                };
                payoff_matrix.push(PayoffMatrixEntry::new(
                    skinny_seq_pl1,
                    skinny_seq_pl2,
                    payoff_entry.chance_factor * leader_chance_factor,
                    payoff_entry.payoff_pl1,
                    payoff_entry.payoff_pl2,
                ));
                // println!("AAAAAAAAAAAA {:?}", payoff_matrix.last());
                
            }
            
        }

        let p = PayoffMatrix::new(payoff_matrix);
        p
    }

    /// Extract infosets relevant to a given subgame and return vector
    /// of bools.
    fn relevant_infosets(&self, player: Player, subgame_id: usize) -> Vec<bool> {
        let treeplex = self.game.treeplex(player);
        let mut is_relevant_infoset = std::vec::from_elem::<bool>(false, treeplex.num_infosets());

        let subgame = SubgameOrFree::Subgame(subgame_id);
        for infoset_id in (0..treeplex.num_infosets())
            .into_iter()
            .filter(|infoset_id| subgame == self.game.subgame(player, *infoset_id))
        {
            is_relevant_infoset[infoset_id] = true;
        }
        is_relevant_infoset
    }

    /// We expand a player's treeplex starting from the empty sequence.
    /// While doing so, we perform 2 things:
    /// (I) Record the head information set(s) in each subgame.
    /// (II) For each sequence, store which *infoset* is the head
    /// of the subgame the sequence belongs to (or none, if it is not a subgame).
    /// We return a 2-tuple for (I) and (II) respectively, in the form of
    /// a Vector of Vector (containing head infosets), for each subgame, and
    /// a Vector (of size num_sequences), containing the head information set
    /// (or None).
    fn preprocess_treeplex_subgames(
        game: &ExtensiveFormGame,
        player: Player,
        treeplex_tools: &TreeplexTools,
    ) -> (Vec<Vec<usize>>, Vec<Option<usize>>) {
        let treeplex = game.treeplex(player);
        let empty_sequence = treeplex.empty_sequence_id();

        let mut heads_per_subgame = std::vec::from_elem::<Vec<usize>>(vec![], game.num_subgames());
        let mut seq_to_head = std::vec::from_elem::<Option<usize>>(None, treeplex.num_sequences());

        // Run top down DFS of the treeplex.
        let mut sequence_stack = Vec::<SequenceId>::new();
        sequence_stack.push(empty_sequence);
        while sequence_stack.len() > 0 {
            let sequence = sequence_stack.pop().unwrap(); // Guaranteed nonempty.

            for next_infoset_id in treeplex_tools.seq_to_infoset_range(sequence) {
                let infoset = treeplex.infosets()[next_infoset_id];
                match game.subgame(player, next_infoset_id) {
                    SubgameOrFree::Subgame(x) => {
                        // Store infoset as a possible head of this subgame.
                        heads_per_subgame[x].push(next_infoset_id);

                        // Iterate over all *descendent* sequences of this
                        // infoset and mark them as being beneath this infoset.
                        let (start_desc_seq, end_desc_seq) =
                            treeplex_tools.seqs_under_infoset(next_infoset_id);
                        for desc_seq in start_desc_seq..=end_desc_seq {
                            seq_to_head[desc_seq] = Some(next_infoset_id);
                        }
                    }
                    SubgameOrFree::Free => {
                        // Recurse with all children sequences.
                        for next_seq in infoset.start_sequence..=infoset.end_sequence {
                            sequence_stack.push(next_seq);
                        }
                    }
                }
            }
        }

        (heads_per_subgame, seq_to_head)
    }
}

