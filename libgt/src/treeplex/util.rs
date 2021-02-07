/// A `Sequence` object is just a 2-tuple of (infoset_index, action_index).
/// Note that `Sequence` does not contain `PlayerInfo` or `Action`, it merely
/// contains *indices* to them.
/// (TODO(chunkail): either refactor or change to reference counted scheme?
/// Maybe rename this to `SequenceIndex` instead.
pub type Sequence = (usize, usize);

/// Specifies a sequence (in terms of `PlayerInfo`, `Action` *index* pairs) or whether this is
/// an empty sequence. This representation is cheap and copy-able at
/// constant time.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub enum SequenceOrEmpty {
    Empty,
    Sequence(Sequence),
}