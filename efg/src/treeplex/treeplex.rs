use crate::game::Infoset;
use crate::game::Player;

use crate::strategy::{BehavioralStrategy, SequenceFormStrategy};
use crate::vector::TreeplexVector;

use crate::schema::game_capnp;
use capnp;

pub type SequenceId = usize;

#[derive(Debug, Clone)]
pub struct Treeplex {
    player: Player,
    num_sequences: usize,
    infosets: Box<[Infoset]>,

    // TODO: remove? Already in utility.
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
}

impl Treeplex {
    pub fn new(player: Player, num_sequences: usize, infosets: Box<[Infoset]>) -> Treeplex {
        Treeplex {
            player,
            num_sequences,
            infosets,
            seq_to_infoset_id_range: vec![],
        }
    }

    pub fn seq_to_infoset_range(&self, sequence_id: usize) -> std::ops::Range<usize> {
        assert!(
            sequence_id < self.seq_to_infoset_id_range.len(),
            "Sequence to infoset range not appropriately initialized yet!"
        );
        self.seq_to_infoset_id_range[sequence_id].clone()
    }

    // Fills in the seq_to_infoset mapping. Should only be called once *if needed*.
    // Performs a bpttom-up traversal if needed.
    pub fn fill_seq_to_infoset_range(&mut self) {
        self.seq_to_infoset_id_range
            .resize(self.num_sequences, 0..0);
        self.seq_to_infoset_id_range.clear();

        let mut cur_sequence = self.num_sequences(); // Initialized to unobtainable value.
        let mut min_infoset_id = self.num_infosets(); // Initialize to unobtainable value.
        for infoset_id in 0..self.num_infosets() {
            let infoset = self.infosets[infoset_id];
            let parent_sequence = infoset.parent_sequence;
            if parent_sequence != cur_sequence {
                // Add infoset range into parent sequence.
                if infoset_id != 0 {
                    self.seq_to_infoset_id_range[cur_sequence] = min_infoset_id..infoset_id;
                }
            } else {
                min_infoset_id = infoset_id;
                cur_sequence = parent_sequence;
            }
        }
        // Add infoset range for final block of infosets.
        self.seq_to_infoset_id_range[cur_sequence] = min_infoset_id..self.num_infosets();
    }

    pub fn num_sequences(&self) -> usize {
        self.num_sequences
    }

    pub fn num_infosets(&self) -> usize {
        self.infosets.len()
    }

    pub fn empty_sequence_id(&self) -> SequenceId {
        self.num_sequences - 1
    }

    pub fn has_sequence(&self, index: SequenceId) -> bool {
        index < self.num_sequences
    }

    pub fn infosets(&self) -> &Box<[Infoset]> {
        &self.infosets
    }

    pub fn player(&self) -> Player {
        self.player
    }

    /// Computes inplace the best *behavioral* response given the *sequence form*
    /// payoff vector in `gradient`.
    pub fn inplace_behavioral_br<'a>(
        &self,
        mut gradient: TreeplexVector<'a>,
    ) -> (f64, BehavioralStrategy<'a>) {
        let best_response_value = self._inplace_behavioral_br(&mut gradient);
        let behavioral_strategy = BehavioralStrategy::from_treeplex_vector(gradient);

        (best_response_value, behavioral_strategy)
    }

    /// Computes inplace the best *sequence* response given the *sequence form*
    /// payoff vector in `gradient`.
    pub fn inplace_sequence_form_br<'a>(
        &self,
        mut gradient: TreeplexVector<'a>,
    ) -> (f64, SequenceFormStrategy<'a>) {
        // To compute the sequence form best response, we first compute the
        // behavioral form best response.
        let best_response_value = self._inplace_behavioral_br(&mut gradient);
        let behavioral_strategy = BehavioralStrategy::from_treeplex_vector(gradient);

        // Now, we convert to the sequence form best response using the methods
        // already defined in `SequenceFormStrategy`.
        let sequence_form_strategy =
            SequenceFormStrategy::from_behavioral_strategy(behavioral_strategy);

        (best_response_value, sequence_form_strategy)
    }

    /// Computes the best *behavioral form*  response given the *sequence form*
    /// payoff vector in `gradient`.
    /// TODO(chunkail): No need for cloning and runnign inplace_xxx_br?
    pub fn behavioral_br<'a>(&self, gradient: TreeplexVector<'a>) -> (f64, BehavioralStrategy<'a>) {
        let br = gradient.clone();
        self.inplace_behavioral_br(br)
    }

    /// Computes the best *sequence form*  response given the *sequence form*
    /// payoff vector in `gradient`.
    /// TODO(chunkail): No need for cloning and cloning inplace_xxx_br?
    pub fn sequence_form_br<'a>(
        &self,
        gradient: TreeplexVector<'a>,
    ) -> (f64, SequenceFormStrategy<'a>) {
        let br = gradient.clone();
        self.inplace_sequence_form_br(br)
    }

    /// Internal function to compute the best behavior best response, which is
    /// returend as a `TreeplexVector`.
    fn _inplace_behavioral_br(&self, gradient: &mut TreeplexVector) -> f64 {
        assert_eq!(gradient.entries.len(), self.num_sequences());

        for infoset_id in 0..self.num_infosets() {
            let infoset = self.infosets[infoset_id];
            let parent_sequence = infoset.parent_sequence;
            let mut best_value = std::f64::NEG_INFINITY;
            let mut best_index = infoset.start_sequence;
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                if best_value < gradient[sequence_id] {
                    best_value = gradient[sequence_id];
                    best_index = sequence_id
                }
            }
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                gradient[sequence_id] = 0.0;
            }
            gradient[best_index] = 1.0;
            gradient[parent_sequence] += best_value;
        }

        let best_response_value = gradient[self.empty_sequence_id()];
        gradient[self.empty_sequence_id()] = 1.0;

        best_response_value
    }

    /// Serializes the treeplex as a Cap'n'proto structure.
    pub fn serialize<'b>(&self, builder: &mut game_capnp::treeplex::Builder<'b>) {
        let mut infosets_builder = builder.reborrow().init_infosets(self.num_infosets() as u32);

        for (infoset_index, infoset) in self.infosets.iter().enumerate() {
            let mut infoset_builder = infosets_builder.reborrow().get(infoset_index as u32);
            infoset.serialize(&mut infoset_builder);
        }

        builder.set_num_sequences(self.num_sequences() as u32);
    }

    /// Deserializes the treeplex from a Cap'n'proto structure.
    pub fn deserialize<'b>(
        player: Player,
        reader: &game_capnp::treeplex::Reader<'b>,
    ) -> capnp::Result<Treeplex> {
        Ok(Treeplex::new(
            player,
            reader.get_num_sequences() as usize,
            {
                let mut infosets = vec![];
                for infoset in reader.get_infosets()?.iter() {
                    infosets.push(Infoset::deserialize(&infoset));
                }
                infosets.into_boxed_slice()
            },
        ))
    }
}

#[cfg(test)]
pub mod test_fixtures {
    use crate::game::Infoset;
    use crate::game::Player;
    use crate::treeplex::Treeplex;
    use crate::vector::TreeplexVector;
    use assert_approx_eq::assert_approx_eq;
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref KUHN_TREEPLEX_PL1: Treeplex = Treeplex::new(
            Player::Player1,
            13,
            vec![
                Infoset::new(6, 0, 1),
                Infoset::new(8, 2, 3),
                Infoset::new(10, 4, 5),
                Infoset::new(12, 6, 7),
                Infoset::new(12, 8, 9),
                Infoset::new(12, 10, 11)
            ]
            .into_boxed_slice()
        );

        pub static ref KUHN_TREEPLEX_PL2: Treeplex = Treeplex::new(
            Player::Player2,
            13,
            vec![
                Infoset::new(12, 0, 1),
                Infoset::new(12, 2, 3),
                Infoset::new(12, 4, 5),
                Infoset::new(12, 6, 7),
                Infoset::new(12, 8, 9),
                Infoset::new(12, 10, 11),
            ]
            .into_boxed_slice()
        );
    }

    #[test]
    fn inplace_best_response() {
        let gradient = &(0..13).rev().map(|x| x as f64).collect::<Vec<f64>>()[..];
        let vector = TreeplexVector::from_array(&KUHN_TREEPLEX_PL1, gradient);
        let (value_br, behavioral_br) = KUHN_TREEPLEX_PL1.inplace_behavioral_br(vector);

        assert_approx_eq!(
            (behavioral_br.inner()
                - &TreeplexVector::from_vec(
                    &KUHN_TREEPLEX_PL1,
                    [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0].to_vec()
                ))
                .max_norm(),
            0.0
        );
        assert_approx_eq!(value_br, 42.0);

        let gradient = &[1., 2., 3., 0., 1., 2., 3., 4., 1., 2., 3., 8., 0.];
        let vector = TreeplexVector::from_array(&KUHN_TREEPLEX_PL1, gradient);
        let (value_br, sequence_form_br) = KUHN_TREEPLEX_PL1.inplace_sequence_form_br(vector);
        println!("{:?}", sequence_form_br);
        assert_approx_eq!(
            (sequence_form_br.inner()
                - &TreeplexVector::from_vec(
                    &KUHN_TREEPLEX_PL1,
                    [0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0].to_vec()
                ))
                .max_norm(),
            0.0
        );
        assert_approx_eq!(value_br, 17.0);
    }
}

#[cfg(test)]
mod tests {

    /*
    #[test]
    fn test_
    */
}