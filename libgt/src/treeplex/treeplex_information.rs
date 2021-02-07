use efg_lite::game::{Infoset, SubgameOrFree};
use efg_lite::treeplex::SequenceId;
use std::collections::{BTreeMap, BTreeSet};

use crate::game_tree::{GameTreeVertex, Leaf};
use crate::treeplex::{SequenceOrEmpty};

/// Unlike `Leaf`, `LeafInfo` comprises `Leaf` and other important information
/// accumulated while traversing the game tree. This intermediate structure is
/// used to temporarily store imporant information about leaves needed to
/// reconstruct treeplex represenations of the game.
#[derive(Debug, Clone)]
pub struct LeafInfo {
    prev_seq_pl1: SequenceOrEmpty,
    prev_seq_pl2: SequenceOrEmpty,
    leaf: Leaf,
    chance_factor: f64,
}

impl LeafInfo {
    pub fn new(
        prev_seq_pl1: SequenceOrEmpty,
        prev_seq_pl2: SequenceOrEmpty,
        leaf: Leaf,
        chance_factor: f64,
    ) -> LeafInfo {
        LeafInfo {
            prev_seq_pl1,
            prev_seq_pl2,
            leaf,
            chance_factor,
        }
    }

    pub fn prev_seq_pl1(&self) -> SequenceOrEmpty {
        self.prev_seq_pl1
    }
    pub fn prev_seq_pl2(&self) -> SequenceOrEmpty {
        self.prev_seq_pl2
    }

    pub fn leaf(&self) -> Leaf {
        self.leaf
    }

    pub fn chance_factor(&self) -> f64 {
        self.chance_factor
    }
}

/// Struct to accumulate the treeplex information for each player while traversing the game
/// tree.
#[derive(Debug, Clone)]
pub struct TreeplexInformation<T: GameTreeVertex> {
    // Storage for infosets, actions, and subgames, and their ids, represented by natural numbers.
    infosets: BTreeMap<T::PlayerInfo, usize>,
    actions: BTreeMap<T::Action, usize>,
    // Maps from infoset (ids) to children sequences.
    infoset_idx_to_seqs: BTreeMap<usize, Vec<SequenceOrEmpty>>,
    // Maps from sequence (2-tuples or empty) to successor infoset (ids).
    seq_to_infoset_idxs: BTreeMap<SequenceOrEmpty, BTreeSet<usize>>,
    // Maps from infoset (ids) to subgame (ids).
    infoset_idx_to_subgame_idx: BTreeMap<usize, SubgameOrFree>,
}

impl<T> TreeplexInformation<T>
where
    T: GameTreeVertex,
{
    pub fn new() -> TreeplexInformation<T> {
        TreeplexInformation {
            infosets: BTreeMap::<T::PlayerInfo, usize>::new(),
            actions: BTreeMap::<T::Action, usize>::new(),
            infoset_idx_to_seqs: BTreeMap::<usize, Vec<SequenceOrEmpty>>::new(),
            seq_to_infoset_idxs: BTreeMap::<SequenceOrEmpty, BTreeSet<usize>>::new(),
            infoset_idx_to_subgame_idx: BTreeMap::<usize, SubgameOrFree>::new(),
        }
    }

    /// Renumbers treeplex into one that obeys the requirements needed for `Treeplex`.
    /// We return the following:
    /// a) mapping from *indices* of old infosets to *indices of* new infosets.
    /// b) mapping from `SequenceOrEmpty` objects to sequence indices.
    /// c) a vector of `Infoset` objects using the *new* numbering system, which
    /// may be used to (almost) define a treeplex.
    pub fn renumber(
        &self,
    ) -> (
        BTreeMap<usize, usize>,
        BTreeMap<SequenceOrEmpty, usize>,
        Vec<Infoset>,
        Vec<SubgameOrFree>,
    ) {
        let num_infosets = self.infosets.len();

        // Mappers map from old to new indices, where new indices obey the
        // numbering conventions specified by efg-lite.
        let mut infoset_mapper = BTreeMap::<usize, usize>::new();
        let mut sequence_mapper = BTreeMap::<SequenceOrEmpty, usize>::new();

        // We iniitalize the subgame and infoset lists with dummy `Result`
        // objects at first, and populate them as required.
        let mut infoset_list = vec![Result::<Infoset, ()>::Err(()); num_infosets];
        let mut subgame_list = vec![Result::<SubgameOrFree, ()>::Err(()); num_infosets];

        // By convention, the empty sequence is labelled 0 [we will reverse this later]
        sequence_mapper.insert(SequenceOrEmpty::Empty, 0);
        self.expand_sequence(
            SequenceOrEmpty::Empty,
            &mut infoset_mapper,
            &mut sequence_mapper,
            &mut infoset_list,
            &mut subgame_list,
        );

        // Now we will need to reverse the sequence and infoset ordering for mappings.
        assert_eq!(infoset_mapper.len(), infoset_list.len());
        let num_sequences = sequence_mapper.len();

        // Reverse numbering schemes in the mappers.
        for (_, v) in infoset_mapper.iter_mut() {
            *v = num_infosets - *v - 1;
        }
        for (_, v) in sequence_mapper.iter_mut() {
            *v = num_sequences - *v - 1;
        }

        // Transform result and reverse infoset ordering.
        let infoset_renumbered = infoset_list
            .into_iter()
            .map(|x| {
                let mut y = x.unwrap().clone();
                y.start_sequence = num_sequences - x.unwrap().end_sequence - 1;
                y.end_sequence = num_sequences - x.unwrap().start_sequence - 1;
                y.parent_sequence = num_sequences - x.unwrap().parent_sequence - 1;
                y
            })
            .rev()
            .collect();

        let subgames_renumbered = subgame_list.into_iter().map(|x| x.unwrap().clone()).rev().collect();

        (infoset_mapper, sequence_mapper, infoset_renumbered, subgames_renumbered)
    }

    /// Expand sequence.
    fn expand_sequence(
        &self,
        sequence: SequenceOrEmpty,
        infoset_mapper: &mut BTreeMap<usize, usize>,
        sequence_mapper: &mut BTreeMap<SequenceOrEmpty, usize>,
        infoset_list: &mut Vec<Result<Infoset, ()>>,
        subgame_list: &mut Vec<Result<SubgameOrFree, ()>>,
    ) {
        let children_infosets = self.seq_to_infoset_idxs.get(&sequence).unwrap();
        for children_infoset in children_infosets {
            assert!(
                !infoset_mapper.contains_key(children_infoset),
                "Loop detected. Infoset index (old) {:?} was added while expanding 
            sequence index (old) {:?} when it has already been added.",
                children_infoset,
                sequence
            );
            infoset_mapper.insert(*children_infoset, infoset_mapper.len());
        }

        // Recursively iterate using *old* numbering system.
        for children_infoset in children_infosets {
            self.expand_infoset(
                *children_infoset,
                // *infoset_mapper.get(children_infoset).unwrap(),
                *sequence_mapper.get(&sequence).unwrap(),
                infoset_mapper,
                sequence_mapper,
                infoset_list,
                subgame_list,
            );
        }
    }

    /// Caution: parent_sequence is referring to the new index for sequence (not the 2-tuple)
    /// but infoset_idx is is under the old mapping.
    fn expand_infoset(
        &self,
        infoset_idx: usize,
        parent_sequence: usize,
        infoset_mapper: &mut BTreeMap<usize, usize>,
        sequence_mapper: &mut BTreeMap<SequenceOrEmpty, usize>,
        infoset_list: &mut Vec<Result<Infoset, ()>>,
        subgame_list: &mut Vec<Result<SubgameOrFree, ()>>,
    ) {
        let children_sequences = self.infoset_idx_to_seqs.get(&infoset_idx).unwrap();
        let min_sequence_new = sequence_mapper.len();
        for children_sequence in children_sequences {
            assert!(
                !sequence_mapper.contains_key(children_sequence),
                "Loop detected. Children sequence (old) {:?} was added while expanding 
                infoset index (old) {:?} when it has already been added.",
                children_sequence,
                infoset_idx
            );
            sequence_mapper.insert(*children_sequence, sequence_mapper.len());
        }
        let max_sequence_new = sequence_mapper.len() - 1;

        // Recursively iterate to expand children sequences. Add into infoset_list.
        let new_infoset =
            Self::infoset_with_renumbered(parent_sequence, min_sequence_new, max_sequence_new);

        let new_infoset_idx = infoset_mapper.get(&infoset_idx).unwrap();
        assert!(
            parent_sequence < min_sequence_new && parent_sequence < max_sequence_new,
            "Infoset {:?} (old numbering) has a parent sequence {:?} (new numbering) which is 
                higher than children sequence index (range {:?} - {:?}).",
            infoset_idx,
            parent_sequence,
            min_sequence_new,
            max_sequence_new
        );
        infoset_list[*new_infoset_idx] = Result::Ok(new_infoset);
        subgame_list[*new_infoset_idx] =
            Result::Ok(*self.infoset_idx_to_subgame_idx.get(&infoset_idx).unwrap());

        for children_sequence in children_sequences {
            self.expand_sequence(
                *children_sequence,
                infoset_mapper,
                sequence_mapper,
                infoset_list,
                subgame_list,
            );
        }
    }

    /// Convert renumbered sequence into an Infoset object.
    /// Currently, this simply `recasts' usize into SequenceId types.
    /// The primary difference is that we convert SubgameOrFree by incrementing the
    /// subgame id by 1 if the infoset is indeed in a subgame, and 0 if the infoset
    /// is free.
    fn infoset_with_renumbered(
        parent_sequence: usize,
        min_sequence_new: usize,
        max_sequence_new: usize,
    ) -> Infoset {
        /*
        let subgame_id: usize = match subgame {
            SubgameOrFree::Free => 0,
            SubgameOrFree::Subgame(x) => x + 1,
        };
        */
        Infoset::new(
            parent_sequence as SequenceId,
            min_sequence_new as SequenceId,
            max_sequence_new as SequenceId,
        )
    }

    pub fn contains_infoset(&self, infoset: &T::PlayerInfo) -> bool {
        let contains_infoset = self.infosets.contains_key(infoset);

        // Make sure that if infoset found, the infoset_idx_to_seqs has a nonempty container.
        if contains_infoset {
            assert!(self
                .infoset_idx_to_seqs
                .contains_key(self.infosets.get(&infoset).unwrap()));
        }

        contains_infoset
    }

    pub fn insert_infoset(&mut self, infoset: T::PlayerInfo, subgame: SubgameOrFree) -> usize {
        assert!(!self.contains_infoset(&infoset));
        let new_infoset_idx = self.infosets.len();
        self.infosets.insert(infoset, self.infosets.len());

        assert!(!self.infoset_idx_to_seqs.contains_key(&new_infoset_idx));
        // Insert empty container into infoset_idx_to_seqs as well.
        self.infoset_idx_to_seqs
            .insert(new_infoset_idx, Vec::<SequenceOrEmpty>::new());

        assert!(!self
            .infoset_idx_to_subgame_idx
            .contains_key(&new_infoset_idx));
        self.infoset_idx_to_subgame_idx
            .insert(new_infoset_idx, subgame);

        new_infoset_idx
    }

    pub fn get_infoset_id(&self, infoset: &T::PlayerInfo) -> usize {
        *self.infosets.get(infoset).unwrap()
    }

    pub fn contains_action(&self, action: &T::Action) -> bool {
        self.actions.contains_key(action)
    }

    pub fn insert_action(&mut self, action: T::Action) -> usize {
        assert!(!self.actions.contains_key(&action));
        self.actions.insert(action, self.actions.len());
        self.actions.len() - 1
    }

    pub fn get_action_id(&self, action: &T::Action) -> usize {
        *self.actions.get(action).unwrap()
    }

    pub fn infosets(&self) -> &BTreeMap<T::PlayerInfo, usize> {
        &self.infosets
    }

    pub fn actions(&self) -> &BTreeMap<T::Action, usize> {
        &self.actions
    }

    pub fn contains_sequence_or_empty(&self, sequence: &SequenceOrEmpty) -> bool {
        self.seq_to_infoset_idxs.contains_key(sequence)
    }

    pub fn insert_sequence_or_empty(&mut self, sequence: SequenceOrEmpty) {
        assert!(!self.contains_sequence_or_empty(&sequence));
        self.seq_to_infoset_idxs
            .insert(sequence, BTreeSet::<usize>::new());
    }

    pub fn insert_infoset_under_sequence(
        &mut self,
        sequence: &SequenceOrEmpty,
        infoset_idx: usize,
    ) {
        self.seq_to_infoset_idxs
            .get_mut(sequence)
            .unwrap()
            .insert(infoset_idx);
    }

    pub fn infosets_under_sequence(&self) -> &BTreeMap<SequenceOrEmpty, BTreeSet<usize>> {
        &self.seq_to_infoset_idxs
    }

    pub fn insert_sequence_under_infoset_id(
        &mut self,
        infoset_id: &usize,
        sequence: SequenceOrEmpty,
    ) {
        self.infoset_idx_to_seqs
            .get_mut(infoset_id)
            .unwrap()
            .push(sequence);
    }

    pub fn sequences_under_infoset(&self) -> &BTreeMap<usize, Vec<SequenceOrEmpty>> {
        &self.infoset_idx_to_seqs
    }

    pub fn get_subgame_from_infoset_idx(&self, infoset_idx: usize) -> SubgameOrFree {
        *self.infoset_idx_to_subgame_idx.get(&infoset_idx).unwrap()
    }

    pub fn get_subgame_from_infoset(&self, infoset: &T::PlayerInfo) -> SubgameOrFree {
        let infoset_idx = self.get_infoset_id(infoset);
        self.get_subgame_from_infoset_idx(infoset_idx)
    }
}