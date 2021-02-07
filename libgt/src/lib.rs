// (LIB)rary for (G)ame (T)rees.
// Re-implementation of libgg by Gabriele Farina (gfarina@cs.cmu.edu).
// Our version is more aligned with the game-tree view of extensive form games.
//
// TODO(chunkail): Massive overhaul on the number of clones used.
// For example, we only need to store sequences once.
// Maybe use a numbering system internally instead?
// Ideally, we should not have to require the `Clone` trait in ::PlayerInfo and ::Action.

extern crate efg_lite;
extern crate env_logger;

pub mod game_tree;
pub mod treeplex;

pub use game_tree::{ChanceOrPlayer, GameTreeVertex, VertexOrLeaf, Leaf};
pub use treeplex::GameAnnotations;
pub use treeplex::ExtensiveFormGameBuilder;
pub use treeplex::{SequenceOrEmpty, Sequence};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
