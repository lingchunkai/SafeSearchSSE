extern crate env_logger;
use crate::game_tree::GameTreeVertex;

use crate::treeplex::{SequenceOrEmpty, TreeplexInformation};
use std::collections::{BTreeMap, BTreeSet};

/// Contains annotations for the games, which comprises annotations
/// for each treeplex separately.
#[derive(Debug, Clone)]
pub struct GameAnnotations<T: GameTreeVertex> {
    pub treeplex_annotations_pl1: TreeplexAnnotations<T>,
    pub treeplex_annotations_pl2: TreeplexAnnotations<T>,
}

impl<T> GameAnnotations<T>
where
    T: GameTreeVertex,
{
    pub fn new(
        treeplex_annotations_pl1: TreeplexAnnotations<T>,
        treeplex_annotations_pl2: TreeplexAnnotations<T>,
    ) -> GameAnnotations<T> {
        GameAnnotations {
            treeplex_annotations_pl1,
            treeplex_annotations_pl2,
        }
    }
}

/// Contains annotations for a single treeplex, which includes the
/// annotations for infosets, actions and sequences for that treeplex.
#[derive(Debug, Clone)]
pub struct TreeplexAnnotations<T: GameTreeVertex> {
    pub infoset_annotations: Vec<Option<T::PlayerInfo>>,
    pub action_annotations: Vec<Option<T::Action>>,
    pub sequence_annotations: Vec<Option<(T::PlayerInfo, T::Action)>>,
}

impl<T> TreeplexAnnotations<T>
where
    T: GameTreeVertex,
{
    /// Generate annotations for a treeplex for a single player.
    pub fn new(
        treeplex_info: &TreeplexInformation<T>,
        infoset_mapper: &BTreeMap<usize, usize>,
        sequence_mapper: &BTreeMap<SequenceOrEmpty, usize>,
    ) -> TreeplexAnnotations<T> {
        let infoset_annotations =
            Self::infoset_annotations(treeplex_info.infosets(), infoset_mapper);
        let action_annotations = Self::action_annotations(treeplex_info.actions());
        let sequence_annotations = Self::sequence_annotations(
            treeplex_info.infosets_under_sequence(),
            sequence_mapper,
            infoset_mapper,
            &infoset_annotations,
            &action_annotations,
        );
        TreeplexAnnotations {
            infoset_annotations,
            action_annotations,
            sequence_annotations,
        }
    }

    /// Generate infoset annotations for a single player. Here, it is just a vector
    /// from the new infoset ids to an actual `PlayerInfo` object. An Option is used
    /// instead of the `PlayerInfo` object directly, because of technical reasons.
    /// TODO (chunkail): fix this.
    fn infoset_annotations(
        infosets: &BTreeMap<T::PlayerInfo, usize>,
        infoset_mapper: &BTreeMap<usize, usize>,
    ) -> Vec<Option<T::PlayerInfo>> {
        let mut idx_to_infoset = Vec::<Option<T::PlayerInfo>>::new();
        idx_to_infoset.resize(infosets.len(), Option::<T::PlayerInfo>::None);
        for (infoset, &infoset_idx) in infosets.iter() {
            let &remapped_infoset_idx = infoset_mapper.get(&infoset_idx).unwrap();
            idx_to_infoset[remapped_infoset_idx] = Option::<T::PlayerInfo>::Some(infoset.clone());
        }

        idx_to_infoset
    }

    /// Generate action annotations for a single player. Here, it is just a vector
    /// from the action ids to an actual `Action` object. An Option is used
    /// instead of the `Action` object directly, because of technical reasons.
    /// TODO (chunkail): fix this.
    fn action_annotations(actions: &BTreeMap<T::Action, usize>) -> Vec<Option<T::Action>> {
        let mut idx_to_action = Vec::<Option<T::Action>>::new();
        idx_to_action.resize(actions.len(), Option::<T::Action>::None);
        for (action, &action_idx) in actions.iter() {
            idx_to_action[action_idx] = Option::<T::Action>::Some(action.clone());
        }

        idx_to_action
    }

    /// This method maps a sequence id (a single number) to an (`PlayerInfo`, `Action`) pair.
    /// An Option is used instead of the the tuple directly, because of technical reasons.
    /// TODO (chunkail): fix this.
    fn sequence_annotations(
        sequence_to_infoset_idx: &BTreeMap<SequenceOrEmpty, BTreeSet<usize>>,
        sequence_mapper: &BTreeMap<SequenceOrEmpty, usize>,
        infoset_mapper: &BTreeMap<usize, usize>,
        new_infoset_id_to_infoset_desc: &Vec<Option<T::PlayerInfo>>,
        new_action_id_to_action_desc: &Vec<Option<T::Action>>,
    ) -> Vec<Option<(T::PlayerInfo, T::Action)>> {
        let mut idx_to_sequence = Vec::<_>::new();
        idx_to_sequence.resize(sequence_to_infoset_idx.len(), Option::<_>::None);
        for (&sequence, _) in sequence_to_infoset_idx.iter() {
            // At this point, sequence is referring to the old orderings for sequence and actions!
            let &remapped_sequence_idx = sequence_mapper.get(&sequence).unwrap();
            let z: Option<(T::PlayerInfo, T::Action)> = match sequence {
                SequenceOrEmpty::Empty => None,
                SequenceOrEmpty::Sequence(s) => {
                    let new_infoset_id = infoset_mapper.get(&s.0.clone()).unwrap().clone();
                    let t: T::PlayerInfo = new_infoset_id_to_infoset_desc
                        .get(new_infoset_id)
                        .unwrap()
                        .clone()
                        .unwrap()
                        .clone();
                    let a: T::Action = new_action_id_to_action_desc
                        .get(s.1.clone())
                        .unwrap()
                        .clone()
                        .unwrap()
                        .clone();
                    Some((t, a))
                }
            };
            idx_to_sequence[remapped_sequence_idx] = z;
        }
        idx_to_sequence
    }
}