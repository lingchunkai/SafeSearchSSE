use crate::strategy::BehavioralStrategy;
use crate::vector::TreeplexVector;
use crate::treeplex::Treeplex;
use assert_approx_eq::assert_approx_eq;

const THRESHOLD_ACCURACY: f64 = 1e-6;

/// `SequenceFormStrategy` is a specialized `TreeplexVector` with some additional functionality.
#[derive(Debug, Clone)]
pub struct SequenceFormStrategy<'a>(TreeplexVector<'a>);
impl<'a> SequenceFormStrategy<'a> {
    pub fn from_treeplex_vector(vector: TreeplexVector<'a>) -> SequenceFormStrategy {
        SequenceFormStrategy::validate(&vector);
        SequenceFormStrategy(vector)
    }

    pub fn from_behavioral_strategy(
        behavioral_strategy: BehavioralStrategy<'a>,
    ) -> SequenceFormStrategy {
        // Perform converstion from behavioral to sequence form in-place via top down traversal
        // of the treeplex.
        let mut vector = behavioral_strategy.into_inner();
        for infoset_id in (0..vector.treeplex().num_infosets()).rev() {
            let infoset = vector.treeplex().infosets()[infoset_id];
            let parent_sequence_mass = vector[infoset.parent_sequence];

            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                vector[sequence_id] *= parent_sequence_mass;
            }
        }

        SequenceFormStrategy::validate(&vector);
        SequenceFormStrategy(vector)

    }

    /// Validate if the given vector is a legitimate sequence-form strategy. Panics upon failure.
    pub fn validate(vector: &TreeplexVector<'a>) {
        assert_approx_eq!(vector.empty_sequence_value(), 1.0, THRESHOLD_ACCURACY);
        for infoset_id in 0..vector.treeplex().num_infosets() {
            let infoset = vector.treeplex().infosets()[infoset_id];
            let mut total_mass = 0f64;
            for sequence_id in infoset.start_sequence..=infoset.end_sequence {
                total_mass += vector[sequence_id];
            }
            let parent_mass = vector[infoset.parent_sequence];
            // println!("Masses do not match. Parent sequence {:?}, Infoset id {:?}", 
            // infoset.parent_sequence, infoset_id);
            // println!("{:?}", vector.treeplex().num_infosets());
            assert_approx_eq!(total_mass - parent_mass, 0f64, THRESHOLD_ACCURACY);

        }

    }

    pub fn from_uniform_strategy(treeplex: &'a Treeplex) -> SequenceFormStrategy<'a>{
        Self::from_behavioral_strategy(BehavioralStrategy::from_uniform_strategy(treeplex))
    }

    pub fn into_inner(self) -> TreeplexVector<'a> {
        self.0
    }

    pub fn inner(&self) -> &TreeplexVector<'a> {
        &self.0
    }
}