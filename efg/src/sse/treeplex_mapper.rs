use crate::treeplex::{SequenceId, Treeplex, TreeplexTools};

/// Structure which gives mapping from original treeplexes to the skinny treeplex.
/// TODO (chunkail): For sequences, we need not store a full map.
/// Instead, all we need to do is store a single integer
/// offset, which is positive if we are mapping from the skinny_treeplex to
/// full treeplex---i.e., we need to add this offset to map back to the original treeplex.
/// If mapping to the skinny treeplex, then the offset will be negative, i.e., we need
/// to subtract the offset from the originaltreeple.
/// TODO (chunkail): Abstract away details for the mapper by modify treeplex mapper incremenatally,
/// as opposed to feed in the  actual vectors (or whatever underlying data structures).
#[derive(Debug)]
pub struct TreeplexMapper {
    seq_to_skinny_seq: Vec<Option<SequenceId>>,

    // Mapping from skinny sequence id to the original sequence id. This does not include
    // the empty skinny sequence, which could be mapped to many sequences, especially those
    // outside of the subgame in question.
    skinny_seq_to_seq: Vec<SequenceId>,

    infoset_to_skinny_infoset: Vec<Option<usize>>,
    skinny_infoset_to_infoset: Vec<usize>,
}

impl TreeplexMapper {
    pub fn new(treeplex: &Treeplex, is_relevant_infoset: &Vec<bool>) -> TreeplexMapper {
        // Compute mappings based on the relevant infosets.
        let (seq_to_skinny_seq, skinny_seq_to_seq) =
            Self::sequence_mapping(treeplex, &is_relevant_infoset);
        let (infoset_to_skinny_infoset, skinny_infoset_to_infoset) =
            Self::infoset_mapping(treeplex, &is_relevant_infoset, &|x| -> bool {
                seq_to_skinny_seq[x].is_some()
            });

        /*
        println!("Is relevant infoset: {:?}", is_relevant_infoset);
        println!("Seq to skinny seq {:?}", seq_to_skinny_seq);
        println!("Skiny seq to seq {:?}", skinny_seq_to_seq);
        println!("Infoset to skinny infoset {:?}", infoset_to_skinny_infoset);
        */

        TreeplexMapper {
            seq_to_skinny_seq,
            skinny_seq_to_seq,
            infoset_to_skinny_infoset,
            skinny_infoset_to_infoset,
        }
    }

    /// Gets sequence mapping between skinny and original sequences.
    fn sequence_mapping(
        treeplex: &Treeplex,
        is_relevant_infoset: &Vec<bool>,
    ) -> (Vec<Option<SequenceId>>, Vec<SequenceId>) {
        let mut is_relevant_sequence = std::vec::from_elem::<bool>(false, treeplex.num_sequences());

        for infoset_id in 0..treeplex.num_infosets() {
            let infoset = treeplex.infosets()[infoset_id];

            if is_relevant_infoset[infoset_id] {
                for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                    is_relevant_sequence[sequence_id] = true;
                }
            }
        }

        // Construct for sequence mappings in both directions.
        let mut seq_to_skinny_seq =
            std::vec::from_elem::<Option<SequenceId>>(None, treeplex.num_sequences());
        let mut skinny_seq_to_seq = Vec::<SequenceId>::new();
        for sequence_id in (0..treeplex.num_sequences())
            .into_iter()
            .filter(|sequence_id| is_relevant_sequence[*sequence_id])
        {
            seq_to_skinny_seq[sequence_id] = Some(skinny_seq_to_seq.len());
            skinny_seq_to_seq.push(sequence_id);
        }

        (seq_to_skinny_seq, skinny_seq_to_seq)
    }

    /// Get infoset mapping between skinny and original infosets.
    /// Warning: we *cannot* simply use infosets in the same order, as
    /// that would violate the constraint that infosets with the same parent
    /// sequence be a contingous range. This would not hold true for the
    /// head infosets, which in the skinny treeplex have the empty sequence
    /// as the parent sequence.
    fn infoset_mapping(
        treeplex: &Treeplex,
        is_relevant_infoset: &Vec<bool>,
        is_relevant_sequence: &Fn(SequenceId) -> bool,
    ) -> (Vec<Option<usize>>, Vec<usize>) {
        let mut infoset_to_skinny_infoset =
            std::vec::from_elem::<Option<usize>>(None, treeplex.num_infosets());
        let mut skinny_infoset_to_infoset = Vec::<usize>::new();

        // Temporary storage for head infosets, we will handle them separately.
        let mut temp_head_infoset_ids = Vec::<usize>::new();

        // We iterate over all relevant infosets, bottom up. If the infosets are
        // not head infosets, we add them to a separate list to be handled at the
        // end.
        for infoset_id in (0..treeplex.num_infosets())
            .into_iter()
            .filter(|infoset_id| is_relevant_infoset[*infoset_id])
        {
            let infoset = treeplex.infosets()[infoset_id];
            match is_relevant_sequence(infoset.parent_sequence) {
                false => {
                    temp_head_infoset_ids.push(infoset_id);
                }
                true => {
                    infoset_to_skinny_infoset[infoset_id] = Some(skinny_infoset_to_infoset.len());
                    skinny_infoset_to_infoset.push(infoset_id);
                }
            }
        }

        // Handle all the head infosets so that they will have continguous infoset indices.
        for infoset_id in temp_head_infoset_ids.into_iter() {
            infoset_to_skinny_infoset[infoset_id] = Some(skinny_infoset_to_infoset.len());
            skinny_infoset_to_infoset.push(infoset_id);
        }

        (infoset_to_skinny_infoset, skinny_infoset_to_infoset)
    }


    pub fn skinny_seq_to_seq(&self, skinny_seq_id: SequenceId) -> SequenceId {
        self.skinny_seq_to_seq[skinny_seq_id]
    }

    pub fn seq_to_skinny_seq(&self, seq_id: SequenceId) -> SequenceId {
        assert!(
            self.seq_to_skinny_seq[seq_id].is_some(),
            "Sequence {:?} not found",
            seq_id
        );
        self.seq_to_skinny_seq[seq_id].unwrap()
    }

    pub fn infoset_to_skinny_infoset(&self, infoset_id: usize) -> usize {
        assert!(
            self.infoset_to_skinny_infoset[infoset_id].is_some(),
            "Infoset {:?} not found",
            infoset_id
        );
        self.infoset_to_skinny_infoset[infoset_id].unwrap()
    }

    pub fn skinny_infoset_to_infoset(&self, skinny_infoset_id: usize) -> usize {
        self.skinny_infoset_to_infoset[skinny_infoset_id]
    }

    pub fn is_sequence_mapped(&self, seq_id: SequenceId) -> bool {
        match self.seq_to_skinny_seq[seq_id] {
            Some(_) => true,
            None => false,
        }
    }

    pub fn is_infoset_mapped(&self, infoset_id: usize) -> bool {
        match self.infoset_to_skinny_infoset[infoset_id] {
            Some(_) => true,
            None => false,
        }
    }

    /// Number of sequences in skinny representation of the treeplex,
    /// *including* the empty sequence.
    pub fn num_skinny_sequences(&self) -> usize {
        self.skinny_seq_to_seq.len() + 1
    }

    pub fn num_skinny_infosets(&self) -> usize {
        self.skinny_infoset_to_infoset.len()
    }

    pub fn skinny_empty_seq(&self) -> usize {
        self.num_skinny_sequences() - 1
    }
}