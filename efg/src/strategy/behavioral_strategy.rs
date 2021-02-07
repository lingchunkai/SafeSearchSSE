use crate::strategy::SequenceFormStrategy;
use crate::treeplex::Treeplex;
use crate::vector::TreeplexVector;

use assert_approx_eq::assert_approx_eq;
const THRESHOLD_ACCURACY: f64 = 1e-6;
const EFFECTIVELY_ZERO: f64 = 1e-6;

/// `BehavioralStrategy` is a specialized `TreeplexVector` with some additional functionality.
#[derive(Debug, Clone)]
pub struct BehavioralStrategy<'a>(TreeplexVector<'a>);
impl<'a> BehavioralStrategy<'a> {
    pub fn from_treeplex_vector(vector: TreeplexVector<'a>) -> BehavioralStrategy {
        BehavioralStrategy::validate(&vector);
        BehavioralStrategy(vector)
    }

    /// Validate if the given vector is a legitimate behavioral strategy. Panics upon failure.
    pub fn validate(vector: &TreeplexVector<'a>) {
        for infoset_id in 0..vector.treeplex().num_infosets() {
            let infoset = vector.treeplex().infosets()[infoset_id];
            let mut total_mass = 0f64;
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                total_mass += vector[sequence_id];
            }
            assert_approx_eq!(total_mass, 1.0, THRESHOLD_ACCURACY);
        }
        let empty_sequence_id = vector.treeplex().empty_sequence_id();
        assert_approx_eq!(vector[empty_sequence_id], 1.0, THRESHOLD_ACCURACY);
    }

    pub fn from_sequence_form_strategy(
        sequence_form_strategy: SequenceFormStrategy<'a>,
    ) -> BehavioralStrategy {
        let mut vector = sequence_form_strategy.into_inner();
        for infoset_id in 0..vector.treeplex().num_infosets() {
            let infoset = vector.treeplex().infosets()[infoset_id];
            let parent_sequence_mass = vector[infoset.parent_sequence];

            match parent_sequence_mass >= EFFECTIVELY_ZERO {
                true => {
                    for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                        vector[sequence_id] = vector[sequence_id] / parent_sequence_mass;
                    }
                }
                false => {
                    // If the parent sequence is zero or almost zero, then we just take the behavioral
                    // strategy to be uniform.
                    let num_sequences = infoset.end_sequence - infoset.start_sequence + 1;
                    for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                        vector[sequence_id] = 1.0 / (num_sequences as f64);
                    }
                }
            }
        }

        BehavioralStrategy::validate(&vector);
        BehavioralStrategy(vector)
    }

    pub fn from_uniform_strategy(treeplex: &'a Treeplex) -> BehavioralStrategy {
        let mut vector: TreeplexVector<'a> = TreeplexVector::from_constant(treeplex, -10f64);
        vector[treeplex.empty_sequence_id()] = 1.0;
        for infoset in treeplex.infosets().iter() {
            let num_sequences = infoset.end_sequence - infoset.start_sequence + 1;
            let prob = 1.0f64 / (num_sequences as f64);
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                vector[sequence_id] = prob;
            }
        }
        Self::from_treeplex_vector(vector)
    }

    pub fn into_inner(self) -> TreeplexVector<'a> {
        self.0
    }

    pub fn inner(&self) -> &TreeplexVector<'a> {
        &self.0
    }

}