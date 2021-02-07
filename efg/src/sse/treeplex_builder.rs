use crate::game::Infoset;
use crate::sse::TreeplexMapper;
use crate::treeplex::{SequenceId, Treeplex};

/// Construct treeplex by looking at all marking all relevant sequences and `squashing' them
/// to get new sequence numbers. The order still obeys the requirements of the original treeplex.
/// We do this similarly for information sets.
/// TODO (chunkail): use iterators like filter and filter_map.
/// The current way of doing is is to first construct a vector of bools for relevant sequences,
/// (i.e., all sequences under relevant infosets), and then iterate through all sequences
/// to get the mapping. This is very inefficient and messy.
///

pub struct TreeplexBuilder<'a> {
    treeplex: &'a Treeplex,
}

impl<'a> TreeplexBuilder<'a> {
    pub fn new(treeplex: &'a Treeplex) -> TreeplexBuilder {
        TreeplexBuilder { treeplex }
    }

    pub fn treeplex_from_mapper(&self, treeplex_mapper: &TreeplexMapper) -> Treeplex {
        let num_sequences_skinny = treeplex_mapper.num_skinny_sequences();
        let mut skinny_infosets = Vec::<Infoset>::new();

        /*
        // Construct infoset list based only on relevant sequences.
        for (infoset_id, infoset) in self
            .treeplex
            .infosets()
            .iter()
            .enumerate()
            .filter(|x| treeplex_mapper.is_infoset_mapped((*x).0))
        {
            // Technically, we only need the offset from the start sequence rather than
            // lookup all mappings iteratively, but we do so anyway for a sanity check.
            let offset = {
                let sequence_id = infoset.start_sequence;
                let skinny_sequence_id = treeplex_mapper.seq_to_skinny_seq(sequence_id);
                sequence_id - skinny_sequence_id
            };
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                let skinny_sequence_id = treeplex_mapper.seq_to_skinny_seq(sequence_id);
                assert_eq!(skinny_sequence_id, sequence_id - offset);
            }

            let start_sequence = infoset.start_sequence - offset;
            let end_sequence = infoset.end_sequence - offset;

            // Here, the behavior for the follower and leader are slightly different. For
            // the follower, the parent sequence should always be relevant, by construction.
            // However, the parent sequence for the leader *may not* be relevant, since we
            // in the skinny game, root infosets of subgames have parent sequences equal to the
            // empty sequence, which may not be their original parent. At any rate, the logic
            // here holds since the check always passes for the follower.
            let parent_sequence = match treeplex_mapper.is_sequence_mapped(infoset.parent_sequence)
            {
                true => treeplex_mapper.seq_to_skinny_seq(infoset.parent_sequence), // Mapped parent, if it exists.
                false => treeplex_mapper.num_skinny_sequences() - 1, // Empty sequence.
            };

            assert_eq!(infoset_id, treeplex_mapper.skinny_infoset_to_infoset(skinny_infosets.len()));
            skinny_infosets.push(Infoset::new(parent_sequence, start_sequence, end_sequence));
        }
        */

        for skinny_infoset_id in 0..treeplex_mapper.num_skinny_infosets() {
            let infoset_id = treeplex_mapper.skinny_infoset_to_infoset(skinny_infoset_id);
            let infoset = self.treeplex.infosets()[infoset_id];

            // Technically, we only need the offset from the start sequence rather than
            // lookup all mappings iteratively, but we do so anyway for a sanity check.
            let offset = {
                let sequence_id = infoset.start_sequence;
                let skinny_sequence_id = treeplex_mapper.seq_to_skinny_seq(sequence_id);
                sequence_id - skinny_sequence_id
            };
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                let skinny_sequence_id = treeplex_mapper.seq_to_skinny_seq(sequence_id);
                assert_eq!(skinny_sequence_id, sequence_id - offset);
            }

            let start_sequence = infoset.start_sequence - offset;
            let end_sequence = infoset.end_sequence - offset;

            // Here, the behavior for the follower and leader are slightly different. For
            // the follower, the parent sequence should always be relevant, by construction.
            // However, the parent sequence for the leader *may not* be relevant, since we
            // in the skinny game, root infosets of subgames have parent sequences equal to the
            // empty sequence, which may not be their original parent. At any rate, the logic
            // here holds since the check always passes for the follower.
            let parent_sequence = match treeplex_mapper.is_sequence_mapped(infoset.parent_sequence)
            {
                true => treeplex_mapper.seq_to_skinny_seq(infoset.parent_sequence), // Mapped parent, if it exists.
                false => treeplex_mapper.num_skinny_sequences() - 1, // Empty sequence.
            };

            assert_eq!(
                infoset_id,
                treeplex_mapper.skinny_infoset_to_infoset(skinny_infosets.len())
            );
            skinny_infosets.push(Infoset::new(parent_sequence, start_sequence, end_sequence));

        }

        Treeplex::new(
            self.treeplex.player(),
            num_sequences_skinny,
            skinny_infosets.into_boxed_slice(),
        )
    }

}