use crate::treeplex::{SequenceId, Treeplex};

use std::cmp::{max, min};

#[derive(Debug)]
pub struct TreeplexTools {

    num_sequences: usize,
    num_infosets: usize,

    // In some algorithms, it may be convenient to perform top down traversal
    // of the treeplex. If the option is not equal to `None`, then it will
    // contain a box of equal size to the number of sequences, each element
    // containing a range (possibly empty) of infoset indices that immediately
    // follow from this sequence.
    // Algorithms which do not require the use of this mapping need not bother
    // with this field, and it will remain empty throughout.
    // In the case where such functionality is required, then the user should
    // make a call to `preprocess_seq_to_infosets`, which runs a tree traversal
    // to compute this data structure.
    // Note that since this information is recosnstrructible from the standard
    // fields of `Treeplex`, we do *not* serialize these ranges to file.
    seq_to_infoset_id_range: Vec<std::ops::Range<usize>>,

    // Start and end SequenceIds of sequences which lie beneath each infoset.
    seqs_under_infosets: Vec<(SequenceId, SequenceId)>,
    seqs_under_seqs: Vec<(SequenceId, SequenceId)>,

    // None if sequence the empty sequence, otherwise, infoset id.
    parent_infoset_of_seqs: Vec<Option<usize>>,
}

impl TreeplexTools {
    pub fn new(treeplex: &Treeplex) -> TreeplexTools {
        let mut tools = TreeplexTools {
            num_sequences: treeplex.num_sequences(),
            num_infosets: treeplex.num_infosets(),
            seq_to_infoset_id_range: Vec::<_>::new(),
            seqs_under_infosets: Vec::<_>::new(),
            seqs_under_seqs: Vec::<_>::new(),
            parent_infoset_of_seqs: Vec::<_>::new(),
        };
        tools.fill_seq_to_infoset_range(treeplex);
        tools.fill_seqs_under_infoset_and_seqs(treeplex);
        tools.fill_parent_infoset_of_seqs(treeplex);
        tools
    }

    /// Returns a range of infoset ids immediately following a sequence.
    pub fn seq_to_infoset_range(&self, sequence_id: SequenceId) -> std::ops::Range<usize> {
        assert!(
            self.seq_to_infoset_id_range.len() == self.num_sequences(),
            "Sequence to infoset range not appropriately initialized yet. Call 
            fill_seq_to_infoset_range prior to this."
        );
        assert!(
            sequence_id < self.seq_to_infoset_id_range.len(),
            "Sequence id out of range."
        );
        self.seq_to_infoset_id_range[sequence_id].clone()
    }

    pub fn seqs_under_infoset(&self, infoset_id: usize) -> (SequenceId, SequenceId) {
        assert!(
            self.seqs_under_infosets.len() == self.num_infosets(),
            "Sequences-under-infosets not filled!"
        );
        self.seqs_under_infosets[infoset_id]
    }

    pub fn seqs_under_seq(&self, sequence_id: SequenceId) -> (SequenceId, SequenceId) {
        assert!(
            self.seqs_under_seqs.len() == self.num_sequences(),
            "Sequences-under-sequences not filled!"
        );
        self.seqs_under_seqs[sequence_id]
    }

    /// Returns parent (immediate) infoset of the queried sequence.
    pub fn parent_infoset_of_seq(&self, sequence_id: SequenceId) -> Option<usize> {
        assert!(
            self.parent_infoset_of_seqs.len() == self.num_sequences(),
            "Parent-infoset_of-seq not filled!"
        );
        self.parent_infoset_of_seqs[sequence_id]
    }

    pub fn num_sequences(&self) -> usize {
        self.num_sequences
    }

    pub fn num_infosets(&self) -> usize {
        self.num_infosets
    }

    /// Fills in the seq_to_infoset mapping by performing a bottom up traversal.
    /// Should only be called once *if needed*.
    /// TODO (chunkail) Use take_while using iterator.
    fn fill_seq_to_infoset_range(&mut self, treeplex: &Treeplex) {
        // println!("Num sequences {:?}", treeplex.num_sequences());
        self.seq_to_infoset_id_range
            .resize(treeplex.num_sequences(), 0..0);

        let mut cur_sequence = treeplex.num_sequences(); // Initialized to unobtainable value.
        let mut min_infoset_id = treeplex.num_infosets(); // Initialize to unobtainable value.
        for infoset_id in 0..treeplex.num_infosets() {
            let infoset = treeplex.infosets()[infoset_id];
            let parent_sequence = infoset.parent_sequence;
            if parent_sequence != cur_sequence {
                // There has been a change in parent sequence.
                if cur_sequence != treeplex.num_sequences() {
                    // Ensure we are only filling each sequence once.
                    /*
                    println!(
                        "Sequence head: {:?}, Infoset Id: {:?}, Existing range {:?}",
                        cur_sequence, infoset_id, self.seq_to_infoset_id_range[cur_sequence]
                    );
                    */
                    assert!(self.seq_to_infoset_id_range[cur_sequence].start == 0);
                    assert!(self.seq_to_infoset_id_range[cur_sequence].end == 0);
                    // Add infoset range into parent sequence.
                    self.seq_to_infoset_id_range[cur_sequence] = min_infoset_id..infoset_id;
                    cur_sequence = parent_sequence;
                    min_infoset_id = infoset_id;
                } else {
                    // This is the first infoset that we are seeing.
                    assert_eq!(infoset_id, 0);
                    min_infoset_id = infoset_id;
                    cur_sequence = parent_sequence;
                }
            }
        }
        // Add infoset range for final block of infosets.
        self.seq_to_infoset_id_range[cur_sequence] = min_infoset_id..treeplex.num_infosets();
    }

    /// Fills in seqs_under_seqs and seqs_under_infosets.
    fn fill_seqs_under_infoset_and_seqs(&mut self, treeplex: &Treeplex) {
        assert!(
            self.seqs_under_seqs.len() == 0,
            "Sequences-under-sequences non-empty!"
        );
        assert!(
            self.seqs_under_infosets.len() == 0,
            "Sequences-under-infosets non-empty!"
        );
        // Fill seqs_under_seqs[i] with (i, i)
        for i in 0..treeplex.num_sequences() {
            self.seqs_under_seqs.push((i, i));
        }

        for infoset_id in 0..treeplex.num_infosets() {
            let infoset = treeplex.infosets()[infoset_id];
            let parent_sequence = infoset.parent_sequence;
            let (mut min_seq_under_infoset, mut max_seq_under_infoset) =
                (treeplex.num_sequences(), 0);

            // Update sequence ranges under infosets.
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                min_seq_under_infoset =
                    min(self.seqs_under_seqs[sequence_id].0, min_seq_under_infoset);
                max_seq_under_infoset =
                    max(self.seqs_under_seqs[sequence_id].1, max_seq_under_infoset);
            }
            self.seqs_under_infosets
                .push((min_seq_under_infoset, max_seq_under_infoset));

            // Update sequence ranges under parent sequence.
            let min_seq_under_parent_seq = min(
                self.seqs_under_seqs[parent_sequence].0,
                min_seq_under_infoset,
            );
            let max_seq_under_parent_seq = max(
                self.seqs_under_seqs[parent_sequence].1,
                max_seq_under_infoset,
            );
            self.seqs_under_seqs[parent_sequence] =
                (min_seq_under_parent_seq, max_seq_under_parent_seq);
        }
    }

    fn fill_parent_infoset_of_seqs(&mut self, treeplex: &Treeplex) {
        assert!(
            self.parent_infoset_of_seqs.len() == 0,
            "parent-infoset-of-seqs is non-empty"
        );
        self.parent_infoset_of_seqs
            .resize(treeplex.num_sequences(), None);

        for (infoset_id, infoset) in treeplex.infosets().iter().enumerate() {
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                self.parent_infoset_of_seqs[sequence_id] = Some(infoset_id);
            }
        }
    }
}