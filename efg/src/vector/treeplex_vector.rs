use crate::schema::vector_capnp;
use crate::treeplex::{SequenceId, Treeplex};

use capnp;
use std::ops::{Add, AddAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

/// A vector associated with a treeplex, with length equal to the number of sequences
/// in that treeplex.
#[derive(Debug, Clone)]
pub struct TreeplexVector<'a> {
    treeplex: &'a Treeplex,

    pub entries: Vec<f64>,
}

impl<'a> TreeplexVector<'a> {
    pub fn from_constant(treeplex: &'a Treeplex, c: f64) -> TreeplexVector<'a> {
        TreeplexVector {
            treeplex,
            entries: vec![c; treeplex.num_sequences()],
        }
    }

    /// Converts array into `TreeplexVector`. Note this clones content of the array.
    /// Should not consume `initial_values`.
    pub fn from_array(treeplex: &'a Treeplex, initial_entries: &'a [f64]) -> TreeplexVector<'a> {
        assert_eq!(treeplex.num_sequences(), initial_entries.len());
        TreeplexVector {
            treeplex,
            entries: initial_entries.to_vec(),
        }
    }

    /// Converts array into `TreeplexVector`. Note this clones content of the array.
    /// *Will* consume `initial_values`.
    pub fn from_vec(treeplex: &'a Treeplex, initial_entries: Vec<f64>) -> TreeplexVector<'a> {
        assert_eq!(treeplex.num_sequences(), initial_entries.len());
        TreeplexVector {
            treeplex,
            entries: initial_entries,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn empty_sequence_value(&self) -> f64 {
        self.entries[self.treeplex.empty_sequence_id()]
    }

    pub fn treeplex(&self) -> &'a Treeplex {
        self.treeplex
    }

    pub fn num_sequences(&self) -> usize {
        self.treeplex.num_sequences()
    }

    pub fn inner(&self, other: &Self) -> f64 {
        (0..self.num_sequences()).fold(0f64, |acc, x| acc + self[x] * other[x])
    }

    /// Compute L2-norm of the `TreeplexVector`.
    /// TODO(chunkail) use trait instead?
    pub fn l2_norm(&self) -> f64 {
        self.entries
            .iter()
            .map(|x| x * x)
            .into_iter()
            .sum::<f64>()
            .sqrt()
    }

    /// Compute L1-norm of the `TreeplexVector`.
    /// TODO(chunkail) use trait instead?
    pub fn l1_norm(&self) -> f64 {
        self.entries
            .iter()
            .map(|x| x.abs())
            .into_iter()
            .sum::<f64>()
    }

    /// Compute max-norm of the `TreeplexVector`.
    /// TODO(chunkail) use trait instead?
    pub fn max_norm(&self) -> f64 {
        self.entries
            .iter()
            .map(|x| x.abs())
            .into_iter()
            .fold(-std::f64::INFINITY, f64::max)
    }

    pub fn serialize<'b>(&self, builder: &mut vector_capnp::vector::Builder<'b>) {
        let mut entries_builder = builder.reborrow().init_entries(self.entries.len() as u32);
        for (entry_index, entry) in self.entries.iter().enumerate() {
            entries_builder.set(entry_index as u32, *entry as f64);
        }
    }

    pub fn deserialize(
        vector_reader: &vector_capnp::vector::Reader,
        treeplex: &'a Treeplex,
    ) -> capnp::Result<TreeplexVector<'a>> {
        let mut entries = vec![];
        for entry in vector_reader.get_entries()?.iter() {
            entries.push(entry);
        }

        Ok(Self::from_vec(treeplex, entries))
    }

    pub fn persist<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut message_builder = capnp::message::Builder::new_default();

        let mut vector_builder = message_builder.init_root::<vector_capnp::vector::Builder>();
        self.serialize(&mut vector_builder);

        capnp::serialize::write_message(writer, &message_builder)
    }
}

impl<'a> Index<SequenceId> for TreeplexVector<'a> {
    type Output = f64;

    fn index(&self, index: SequenceId) -> &f64 {
        assert!(self.treeplex().has_sequence(index));
        &self.entries[index]
    }
}
impl<'a> IndexMut<SequenceId> for TreeplexVector<'a> {
    fn index_mut(&mut self, index: SequenceId) -> &mut f64 {
        assert!(self.treeplex().has_sequence(index));
        &mut self.entries[index]
    }
}

impl<'a, 'b> SubAssign<&'b TreeplexVector<'b>> for TreeplexVector<'a> {
    fn sub_assign(&mut self, other: &'b TreeplexVector<'b>) {
        for index in 0..self.num_sequences() {
            self[index] -= other[index];
        }
    }
}

impl<'a, 'b> AddAssign<&'b TreeplexVector<'b>> for TreeplexVector<'a> {
    fn add_assign(&mut self, other: &'b TreeplexVector<'b>) {
        for index in 0..self.num_sequences() {
            self[index] += other[index];
        }
    }
}

impl<'a, 'b> MulAssign<&'b TreeplexVector<'a>> for TreeplexVector<'a> {
    fn mul_assign(&mut self, other: &'b Self) {
        for index in 0..self.num_sequences() {
            self[index] *= other[index];
        }
    }
}

impl<'a, 'b> Sub<&'b TreeplexVector<'a>> for TreeplexVector<'a> {
    type Output = TreeplexVector<'a>;
    fn sub(self, other: &'b Self) -> TreeplexVector<'a> {
        let mut obj = self;
        obj -= other;
        obj
    }
}
impl<'a, 'b> Add<&'b TreeplexVector<'a>> for TreeplexVector<'a> {
    type Output = TreeplexVector<'a>;
    fn add(self, other: &'b Self) -> TreeplexVector<'a> {
        let mut obj = self;
        obj += other;
        obj
    }
}

impl<'a, 'b> Mul<&'b TreeplexVector<'a>> for TreeplexVector<'a> {
    type Output = TreeplexVector<'a>;
    fn mul(self, other: &'b Self) -> TreeplexVector<'a> {
        let mut obj = self;
        obj *= other;
        obj
    }
}

impl<'a, 'b> Sub<&'b TreeplexVector<'a>> for &TreeplexVector<'a> {
    type Output = TreeplexVector<'a>;
    fn sub(self, other: &'b TreeplexVector<'a>) -> TreeplexVector<'a> {
        let mut obj = self.clone();
        obj -= other;
        obj
    }
}

impl<'a, 'b> Add<&'b TreeplexVector<'a>> for &TreeplexVector<'a> {
    type Output = TreeplexVector<'a>;
    fn add(self, other: &'b TreeplexVector<'a>) -> TreeplexVector<'a> {
        let mut obj = self.clone();
        obj += other;
        obj
    }
}

impl<'a, 'b> Mul<&'b TreeplexVector<'a>> for &TreeplexVector<'a> {
    type Output = TreeplexVector<'a>;
    fn mul(self, other: &'b TreeplexVector<'a>) -> TreeplexVector<'a> {
        let mut obj = self.clone();
        obj *= other;
        obj
    }
}

impl<'a> SubAssign<f64> for TreeplexVector<'a> {
    fn sub_assign(&mut self, sub: f64) {
        for index in 0..self.num_sequences() {
            self[index] -= sub;
        }
    }
}
impl<'a> AddAssign<f64> for TreeplexVector<'a> {
    fn add_assign(&mut self, add: f64) {
        for index in 0..self.num_sequences() {
            self[index] += add;
        }
    }
}
impl<'a> MulAssign<f64> for TreeplexVector<'a> {
    fn mul_assign(&mut self, mul: f64) {
        for index in 0..self.num_sequences() {
            self[index] *= mul;
        }
    }
}

impl<'a> Sub<f64> for TreeplexVector<'a> {
    type Output = TreeplexVector<'a>;
    fn sub(self, sub: f64) -> TreeplexVector<'a> {
        let mut obj = self;
        obj -= sub;
        obj
    }
}

impl<'a> Add<f64> for TreeplexVector<'a> {
    type Output = TreeplexVector<'a>;
    fn add(self, add: f64) -> TreeplexVector<'a> {
        let mut obj = self;
        obj += add;
        obj
    }
}

impl<'a> Mul<f64> for TreeplexVector<'a> {
    type Output = TreeplexVector<'a>;
    fn mul(self, mul: f64) -> TreeplexVector<'a> {
        let mut obj = self;
        obj *= mul;
        obj
    }
}

impl<'a> Sub<f64> for &TreeplexVector<'a> {
    type Output = TreeplexVector<'a>;
    fn sub(self, sub: f64) -> TreeplexVector<'a> {
        let mut obj = self.clone();
        obj -= sub;
        obj
    }
}

impl<'a> Add<f64> for &TreeplexVector<'a> {
    type Output = TreeplexVector<'a>;
    fn add(self, add: f64) -> TreeplexVector<'a> {
        let mut obj = self.clone();
        obj += add;
        obj
    }
}

impl<'a> Mul<f64> for &TreeplexVector<'a> {
    type Output = TreeplexVector<'a>;
    fn mul(self, mul: f64) -> TreeplexVector<'a> {
        let mut obj = self.clone();
        obj *= mul;
        obj
    }
}


#[cfg(test)]
pub mod test_fixtures {
    use crate::game::{Infoset, Player};
    use crate::treeplex::Treeplex;
    use crate::vector::TreeplexVector;
    use assert_approx_eq::assert_approx_eq;

    use lazy_static::lazy_static;
    lazy_static! {
        pub static ref CHAIN_TREEPLEX: Treeplex = Treeplex::new(
            Player::Player1,
            4,
            vec![
                Infoset::new(1, 0, 0),
                Infoset::new(2, 1, 1),
                Infoset::new(3, 2, 2),
            ]
            .into_boxed_slice()
        );
    }


    #[test]
    pub fn init() {
        let initial_entries = [1.0, 2.0, 3.0, 4.0];
        let _v1 = TreeplexVector::from_array(&CHAIN_TREEPLEX, &initial_entries);

        // We should still have ownership of `initial_entries` at this point.
        let _v2 = TreeplexVector::from_vec(&CHAIN_TREEPLEX, initial_entries.to_vec());

        let _v3 = TreeplexVector::from_constant(&CHAIN_TREEPLEX, 1.47);

        assert_approx_eq!(_v1[0], 1.0);
        assert_approx_eq!(_v1[3], 4.0);
        assert_approx_eq!(_v2[0], 1.0);
        assert_approx_eq!(_v2[3], 4.0);
        assert_approx_eq!(_v3[0], 1.47);
        assert_approx_eq!(_v3[3], 1.47);
    }

    #[test]
    #[should_panic]
    pub fn init_wrong_size() {
        let wrong_sized_entries = [1.0, 2.0, 3.0];
        TreeplexVector::from_array(&CHAIN_TREEPLEX, &wrong_sized_entries);
    }

    #[test]
    pub fn indexing() {
        let initial_entries = [1.0, 2.0, 3.0, 4.0];
        let mut v = TreeplexVector::from_vec(&CHAIN_TREEPLEX, initial_entries.to_vec());
        assert_approx_eq!(v[0], 1.0);
        v[0] += 1.9;
        assert_approx_eq!(v[0], 2.9);
    }

    #[test]
    #[should_panic]
    pub fn indexing_oob() {
        let v = TreeplexVector::from_constant(&CHAIN_TREEPLEX, 1.47);
        v[4];
    }

    #[test]
    pub fn vector_operations() {
        let initial_entries = [1.0, 2.0, 3.0, 4.0];
        let mut v = TreeplexVector::from_vec(&CHAIN_TREEPLEX, initial_entries.to_vec());
        let mut g = TreeplexVector::from_vec(&CHAIN_TREEPLEX, [4.0, 3.0, 2.0, 1.0].to_vec());
        let h = TreeplexVector::from_vec(&CHAIN_TREEPLEX, [0.0, 1.0, 0.0, 1.0].to_vec());

        // Add, Sub and Mul operations.
        v += &g; // We have to use a borrow here and not hand over ownership.
        assert_approx_eq!(v[0], 5.0);
        assert_approx_eq!(v[1], 5.0);
        assert_approx_eq!(v[2], 5.0);
        assert_approx_eq!(v[3], 5.0);

        v *= &g;
        assert_approx_eq!(v[0], 20.0);
        assert_approx_eq!(v[1], 15.0);
        assert_approx_eq!(v[2], 10.0);
        assert_approx_eq!(v[3], 5.0);

        g -= &v;
        assert_approx_eq!(g[0], -16.0);
        assert_approx_eq!(g[1], -12.0);
        assert_approx_eq!(g[2], -8.0);
        assert_approx_eq!(g[3], -4.0);

        // Add, Sub and Mul operations.
        v = g - &h;
        assert_approx_eq!(v[0], -16.0);
        assert_approx_eq!(v[1], -13.0);
        assert_approx_eq!(v[2], -8.0);
        assert_approx_eq!(v[3], -5.0);

        // We no longer have ownership of g at this point.
        // But we could have avoided losing ownership of g
        // by using instead doing, which creates a clone of v
        // a-priori.
        g = &v + &h;
        assert_approx_eq!(g[0], -16.0);
        assert_approx_eq!(g[1], -12.0);
        assert_approx_eq!(g[2], -8.0);
        assert_approx_eq!(g[3], -4.0);

    }

    #[test]
    pub fn scalar_operations() {
        let mut v = TreeplexVector::from_constant(&CHAIN_TREEPLEX, 1.0);
        v *= 0.0;
        assert_approx_eq!(v[0], 0.0);
        assert_approx_eq!(v[1], 0.0);
        assert_approx_eq!(v[2], 0.0);
        assert_approx_eq!(v[3], 0.0);

        v += &TreeplexVector::from_vec(&CHAIN_TREEPLEX, [1.0, -1.0, 1.0, -1.0].to_vec());
        assert_approx_eq!(v[0], 1.0);
        assert_approx_eq!(v[1], -1.0);
        assert_approx_eq!(v[2], 1.0);
        assert_approx_eq!(v[3], -1.0);

        v = &v + &TreeplexVector::from_vec(&CHAIN_TREEPLEX, [1.0, 1.0, 1.0, 1.0].to_vec());
        assert_approx_eq!(v[0], 2.0);
        assert_approx_eq!(v[1], 0.0);
        assert_approx_eq!(v[2], 2.0);
        assert_approx_eq!(v[3], 0.0);

        v = &v
            - &(&TreeplexVector::from_vec(&CHAIN_TREEPLEX, [1.0, 1.0, 1.0, 1.0].to_vec())
                + &TreeplexVector::from_vec(&CHAIN_TREEPLEX, [1.0, 1.0, 1.0, 1.0].to_vec()));
        assert_approx_eq!(v[0], 0.0);
        assert_approx_eq!(v[1], -2.0);
        assert_approx_eq!(v[2], 0.0);
        assert_approx_eq!(v[3], -2.0);
    }

    #[test]
    pub fn norms() {
        let initial_entries = [1.0, 2.0, -3.0, 4.0];
        let mut v = TreeplexVector::from_vec(&CHAIN_TREEPLEX, initial_entries.to_vec());

        assert_approx_eq!(v.max_norm(), 4.0);
        assert_approx_eq!(v.l2_norm(), f64::sqrt(1.0 + 4.0 + 9.0 + 16.0));
        assert_approx_eq!(v.l1_norm(), 10.0);
    }
}