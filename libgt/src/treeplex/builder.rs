extern crate env_logger;

use crate::efg_lite::game::{
    ExtensiveFormGame, Infoset, PayoffMatrix, PayoffMatrixEntry, Player, SubgameOrFree,
};
use crate::efg_lite::treeplex::Treeplex;
use crate::game_tree::{ChanceOrPlayer, GameTreeVertex, Leaf, VertexOrLeaf};

use crate::treeplex::AuxState;

use crate::treeplex::{GameAnnotations, TreeplexAnnotations};
use crate::treeplex::{LeafInfo, SequenceOrEmpty, TreeplexInformation};
use assert_approx_eq::assert_approx_eq;
use itertools::sorted;
use std::collections::BTreeMap;
use std::rc::Rc;

/// Builder for an extensive form game. The primary purpose of this
/// class is to traverse a game tree, store relevant information regarding treeplexes
/// and the payoff matrix, and eventually construct a `ExtensiveFormGame` object,
/// with the numbering required by `Treeplex`.
pub struct ExtensiveFormGameBuilder<T: GameTreeVertex> {
    // TODO(chunkail): configurations in the future (?)
    treeplex_info_pl1: TreeplexInformation<T>,
    treeplex_info_pl2: TreeplexInformation<T>,

    // Storage for leaf information.
    leaves_information: Vec<LeafInfo>,

    // Mapping from subgames to subgame indices.
    subgames: BTreeMap<T::Subgame, usize>,
}

impl<'a, T> ExtensiveFormGameBuilder<T>
where
    T: GameTreeVertex,
{
    /// Initializes an `ExtensiveFormGameBuilder` and returns it.
    pub fn new() -> ExtensiveFormGameBuilder<T> {
        let mut builder = ExtensiveFormGameBuilder {
            treeplex_info_pl1: TreeplexInformation::new(),
            treeplex_info_pl2: TreeplexInformation::new(),
            leaves_information: Vec::<_>::new(),
            subgames: BTreeMap::<_, _>::new(),
        };

        // Initialize by adding in empty sequence to each treeplex_info object.
        builder
            .treeplex_info_pl1
            .insert_sequence_or_empty(SequenceOrEmpty::Empty);
        builder
            .treeplex_info_pl2
            .insert_sequence_or_empty(SequenceOrEmpty::Empty);

        builder
    }

    /// Creates an `ExtensiveFormGame` by starting a traversal from a specified initial_vertex.
    /// This is the primary function in the `ExtensiveFormGameBuilder` class.
    /// Actions within a given infoset are ordered based on the order specified
    /// by `GameTreeVertex::Action`.
    /// However, we do *not* give any guarantees on the details of how this is achieved
    /// so do make any such assumptions such as actions being ordered in increasing
    /// order of `GameTreeVertex::Action`.
    /// That said, it is safe to assume that this mapping is deterministic.
    /// A similar statement holds for children information set mappings to sequences.
    /// TODO (chunkail): Maybe make self be consumed after calling this method,
    /// since the internals of the builder are already dirty at this point.
    pub fn make_game_and_annotations(
        &mut self,
        initial_vertex: &T,
        include_annotations: bool,
    ) -> (ExtensiveFormGame, Option<GameAnnotations<T>>) {
        self.traverse_tree(initial_vertex);
        let (infoset_mapper_pl1, sequence_mapper_pl1, infoset_list_pl1, subgame_list_pl1) =
            self.process_treeplex_info(Player::Player1);
        let (infoset_mapper_pl2, sequence_mapper_pl2, infoset_list_pl2, subgame_list_pl2) =
            self.process_treeplex_info(Player::Player2);

        // TOOD (chunkail) : subgames.
        let efg = self.generate_efg(
            infoset_list_pl1,
            infoset_list_pl2,
            &sequence_mapper_pl1,
            &sequence_mapper_pl2,
            &subgame_list_pl1,
            &subgame_list_pl2,
        );

        // Create annotations if it was requested for.
        let annotations = match include_annotations {
            true => Some(self.generate_annotations(
                infoset_mapper_pl1,
                infoset_mapper_pl2,
                sequence_mapper_pl1,
                sequence_mapper_pl2,
            )),
            false => None,
        };
        (efg, annotations)
    }

    /// Traverse the tree while storing a) last sequence of each Player,
    /// and b) the cumulative effects of chance until a particular vertex.
    /// This function performs (almost) depth-first tree traversal manually
    /// using a stack.
    fn traverse_tree(&mut self, initial_vertex: &T) {
        let initial_aux_state = AuxState {
            prev_seq_pl1: SequenceOrEmpty::Empty,
            prev_seq_pl2: SequenceOrEmpty::Empty,
            chance_factor: 1.0,
            prev_subgame: SubgameOrFree::Free,
        };

        // Walk over game tree.
        let mut vertex_stack = Vec::<(T, AuxState)>::new();
        vertex_stack.push((initial_vertex.clone(), initial_aux_state.clone()));

        while vertex_stack.len() > 0 {
            let (vertex, aux_state) = vertex_stack.pop().unwrap();
            vertex.validate();
            match vertex.next_player() {
                ChanceOrPlayer::Chance => {
                    self.handle_chance(&vertex, aux_state, &mut vertex_stack);
                }
                ChanceOrPlayer::Player(player) => {
                    self.handle_player(player, &vertex, aux_state, &mut vertex_stack)
                }
            };
        }
    }

    /// Processes the treeplexes generated afgter tree traversal and converts
    /// them to a format amenable to that required by `Treeplex` and `ExtensiveFormGame`.
    fn process_treeplex_info(
        &self,
        player: Player,
    ) -> (
        BTreeMap<usize, usize>,
        BTreeMap<SequenceOrEmpty, usize>,
        Vec<Infoset>,
        Vec<SubgameOrFree>,
    ) {
        match player {
            Player::Player1 => self.treeplex_info_pl1.renumber(),
            Player::Player2 => self.treeplex_info_pl2.renumber(),
        }
    }

    /// Generates an extensive form game given the mappings for information sets
    /// and sequence numbers.
    fn generate_efg(
        &self,
        infoset_list_pl1: Vec<Infoset>,
        infoset_list_pl2: Vec<Infoset>,
        sequence_mapper_pl1: &BTreeMap<SequenceOrEmpty, usize>,
        sequence_mapper_pl2: &BTreeMap<SequenceOrEmpty, usize>,
        subgame_list_pl1: &Vec<SubgameOrFree>,
        subgame_list_pl2: &Vec<SubgameOrFree>,
    ) -> ExtensiveFormGame {
        let num_sequences_pl1 = sequence_mapper_pl1.len();
        let num_sequences_pl2 = sequence_mapper_pl2.len();
        let treeplex_pl1 = Rc::<Treeplex>::new(Treeplex::new(
            Player::Player1,
            num_sequences_pl1,
            infoset_list_pl1.into_boxed_slice(),
        ));
        let treeplex_pl2 = Rc::<Treeplex>::new(Treeplex::new(
            Player::Player2,
            num_sequences_pl2,
            infoset_list_pl2.into_boxed_slice(),
        ));

        // Build payoff matrix using the numbering scheme given by the mappers obtained above.
        let mut entries = Vec::<PayoffMatrixEntry>::new();
        for leaf_info in self.leaves_information.iter() {
            let seq_pl1 = sequence_mapper_pl1.get(&leaf_info.prev_seq_pl1()).unwrap();
            let seq_pl2 = sequence_mapper_pl2.get(&leaf_info.prev_seq_pl2()).unwrap();

            let payoff_matrix_entry = PayoffMatrixEntry::new(
                *seq_pl1,
                *seq_pl2,
                leaf_info.chance_factor(),
                leaf_info.leaf().payoff_pl1,
                leaf_info.leaf().payoff_pl2,
            );
            entries.push(payoff_matrix_entry);
        }
        let payoff_matrix = PayoffMatrix::new(entries);

        ExtensiveFormGame::new(
            treeplex_pl1,
            treeplex_pl2,
            payoff_matrix,
            subgame_list_pl1.clone(),
            subgame_list_pl2.clone(),
        )
    }

    /// Generate game annotations for both players.
    fn generate_annotations(
        &self,
        infoset_mapper_pl1: BTreeMap<usize, usize>,
        infoset_mapper_pl2: BTreeMap<usize, usize>,
        sequence_mapper_pl1: BTreeMap<SequenceOrEmpty, usize>,
        sequence_mapper_pl2: BTreeMap<SequenceOrEmpty, usize>,
    ) -> GameAnnotations<T> {
        let treeplex_annotations_pl1 = TreeplexAnnotations::new(
            &self.treeplex_info_pl1,
            &infoset_mapper_pl1,
            &sequence_mapper_pl1,
        );
        let treeplex_annotations_pl2 = TreeplexAnnotations::new(
            &self.treeplex_info_pl2,
            &infoset_mapper_pl2,
            &sequence_mapper_pl2,
        );
        GameAnnotations::new(treeplex_annotations_pl1, treeplex_annotations_pl2)
    }

    /// Update treeplex information when a new (unseen before) information set is encountered.
    fn update_treeplex_info(
        treeplex_info: &mut TreeplexInformation<T>,
        cur_infoset: &T::PlayerInfo,
        actions_list: &Vec<T::Action>,
        preceding_sequence: SequenceOrEmpty,
        subgame: SubgameOrFree,
    ) {
        if !treeplex_info.contains_infoset(&cur_infoset) {
            // Insert infoset into storage.
            let infoset_idx = treeplex_info.insert_infoset(cur_infoset.clone(), subgame);

            // Insert actions into storage and insert sequences into infoset_to_sequences.
            for action in actions_list {
                // Add action if this was the first time we have encountered it.
                if !treeplex_info.contains_action(&action) {
                    treeplex_info.insert_action(action.clone());
                }
                let action_idx = treeplex_info.get_action_id(&action);

                // Add infoset to the potential child of the preceding sequence.
                // It is OK for this to only be added only when this is a unseen for infoset, since
                // perfect recall implies the preceding sequence for this information set has
                // to be precisely preceding_sequence. If this infoset has not been added before, then
                // this particular sequence-child relationship has not been recorded before, and vice-versa.
                treeplex_info.insert_infoset_under_sequence(&preceding_sequence, infoset_idx);

                // Similarly, we add the sequence to the child of this infoset.
                let new_seq = SequenceOrEmpty::Sequence((infoset_idx, action_idx));
                treeplex_info.insert_sequence_under_infoset_id(&infoset_idx, new_seq);
            }
        } else {
            // Do some checks to ensure subgames are consistent.
            let infoset_idx = treeplex_info.get_infoset_id(&cur_infoset);
            assert_eq!(
                subgame,
                treeplex_info.get_subgame_from_infoset_idx(infoset_idx)
            );
        }
    }

    /// Creates a `LeafInfo` which stores references to sequences and payoffs. Pushes this
    /// onto leaves_information, a structure which stores `LeafInfo`.
    fn handle_leaf(leaves_information: &mut Vec<LeafInfo>, leaf: &Leaf, aux_state: AuxState) {
        let leaf_info = LeafInfo::new(
            aux_state.prev_seq_pl1,
            aux_state.prev_seq_pl2,
            leaf.clone(),
            aux_state.chance_factor,
        );
        leaves_information.push(leaf_info);
    }

    /// Expands a chance node, and based on whether next_state is terminal or a vertex,
    /// either handle the leaf or push the vertex onto the vertex_stack.
    fn handle_chance(
        &mut self,
        vertex: &T,
        aux_state: AuxState,
        vertex_stack: &mut Vec<(T, AuxState)>,
    ) {
        let actions_and_probs = vertex.available_actions();
        let total_prob: f64 = actions_and_probs
            .iter()
            .map(|p: &(T::Action, f64)| p.1)
            .sum();
        assert_approx_eq!(total_prob, 1.0);

        for (action, prob) in vertex.available_actions().into_iter().cloned() {
            assert!(prob >= 0f64);
            let next_vertex_or_leaf = vertex.next_state(&action);

            // Handle accordingly depending on whether the next vertex is
            // another vertex or a leaf state.
            match next_vertex_or_leaf {
                VertexOrLeaf::Vertex(vertex) => {
                    let new_subgame_description = vertex.subgame();
                    Self::check_subgame_consistency(
                        &mut self.subgames,
                        &new_subgame_description,
                        &aux_state.prev_subgame,
                    );
                    let new_subgame = Self::add_subgame_if_needed_and_get(
                        &mut self.subgames,
                        new_subgame_description,
                    );
                    let new_aux_state: AuxState =
                        aux_state.new_with_updated_chance(prob, new_subgame);
                    vertex_stack.push((vertex, new_aux_state))
                }
                VertexOrLeaf::Leaf(leaf) => {
                    let new_subgame = aux_state.prev_subgame.clone(); // Subgame is assumed to be unchanged if leaf.
                    let new_aux_state: AuxState =
                        aux_state.new_with_updated_chance(prob, new_subgame);
                    Self::handle_leaf(&mut self.leaves_information, &leaf, new_aux_state)
                }
            }
        }
    }

    /// Expands a player node, and based on whether next_state is terminal or a vertex,
    /// either handle the leaf or push the vertex onto the vertex_stack. Also, if the information
    /// set is seen for the first time, then we update this information set/ sequence mapping
    /// in the treeplex_info.
    fn handle_player(
        &mut self,
        player: Player,
        vertex: &T,
        aux_state: AuxState,
        vertex_stack: &mut Vec<(T, AuxState)>,
    ) {
        let cur_infoset = vertex.player_information();
        let actions_list: Vec<T::Action> = sorted(
            vertex
                .available_actions()
                .into_iter()
                .map(|p: &(T::Action, f64)| p.0.clone()),
        )
        .collect();

        let (treeplex_info, preceding_sequence) = match player {
            Player::Player1 => (&mut self.treeplex_info_pl1, aux_state.prev_seq_pl1),
            Player::Player2 => (&mut self.treeplex_info_pl2, aux_state.prev_seq_pl2),
        };

        // Check if this infoset was visited before. If not, we iterate over all actions and
        // place it in infoset_to_actions array. Also, we have to add the precending sequence
        // (obtained from aux_state) as preceding this infoset.
        Self::update_treeplex_info(
            treeplex_info,
            &cur_infoset,
            &actions_list,
            preceding_sequence,
            aux_state.prev_subgame,
        );

        // Now we iterate over all actions and get the next state given, and recursively
        // add in future vertices to the stack.
        for action in actions_list {
            let next_vertex_or_leaf = vertex.next_state(&action);
            let new_sequence = (
                treeplex_info.get_infoset_id(&cur_infoset),
                treeplex_info.get_action_id(&action),
            );

            // Add empty vector of children information sets if this is the first time we have
            // encountered this sequence. TODO: (chunkail) use __.entry.or_insert() instead.
            if !treeplex_info.contains_sequence_or_empty(&SequenceOrEmpty::Sequence(new_sequence)) {
                treeplex_info.insert_sequence_or_empty(SequenceOrEmpty::Sequence(new_sequence));
            }

            // Handle accordingly depending on whether the next vertex is another vertex or terminal.
            match next_vertex_or_leaf {
                VertexOrLeaf::Vertex(vertex) => {
                    let new_subgame_description = vertex.subgame();
                    Self::check_subgame_consistency(
                        &mut self.subgames,
                        &new_subgame_description,
                        &aux_state.prev_subgame,
                    );
                    let new_subgame = Self::add_subgame_if_needed_and_get(
                        &mut self.subgames,
                        new_subgame_description,
                    );
                    let new_aux_state: AuxState =
                        aux_state.new_with_updated_sequence(player, new_sequence, new_subgame);
                    vertex_stack.push((vertex, new_aux_state));
                }
                VertexOrLeaf::Leaf(leaf) => {
                    let new_subgame = aux_state.prev_subgame.clone(); // Subgame is assumed to be unchanged if leaf.
                    let new_aux_state: AuxState =
                        aux_state.new_with_updated_sequence(player, new_sequence, new_subgame);
                    Self::handle_leaf(&mut self.leaves_information, &leaf, new_aux_state);
                }
            };
        }
    }

    /// Test if subgame is consistent with parent---if new subgame is free, then
    /// parent must be none too. If both subgames are not free, then they must be
    /// equal.
    fn check_subgame_consistency(
        subgames: &mut BTreeMap<T::Subgame, usize>,
        subgame_description: &Option<T::Subgame>,
        prev_subgame: &SubgameOrFree,
    ) {
        match (subgame_description, prev_subgame) {
            (None, SubgameOrFree::Free) => {} // Both ancestor and descendent do not belong to any subgame---possible.
            (None, SubgameOrFree::Subgame(_)) => {
                // Descendent does not belong to subgame, but ancestor does---impossible.
                panic!("Parent was in a subgame but children was not");
            }
            // Descendent is in some subgame, but ancestor was not---possible.
            (Some(_), SubgameOrFree::Free) => {}
            // Descendent and ancestor belong to some subgame each---possible, but have
            // to check if they belong to the same subgame. To do that, we make sure
            // that (1) new_subgame_desc belongs in the mapping of subgames and
            // (2) what it maps to is equal to what the vertex's ancestor is mapped to.
            (Some(new_subgame_desc), SubgameOrFree::Subgame(idx)) => {
                assert_eq!(subgames.get(&new_subgame_desc).unwrap(), idx);
            }
        }
    }

    fn add_subgame_if_needed_and_get(
        subgames: &mut BTreeMap<T::Subgame, usize>,
        subgame_description: Option<T::Subgame>,
    ) -> SubgameOrFree {
        // Add in subgame if it has not already been added.
        if subgame_description.is_some() {
            let l = subgames.len();
            SubgameOrFree::Subgame(*subgames.entry(subgame_description.unwrap()).or_insert(l))
        } else {
            SubgameOrFree::Free
        }
    }
}