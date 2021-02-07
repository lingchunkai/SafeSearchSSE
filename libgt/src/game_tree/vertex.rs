
use efg_lite::game::Player;
use std::fmt::Debug;
/// `GameTreeVertex` is the main trait that should be implemented by the game generator.
/// The game generator implicitly defines a game tree based on the implemented functions.
pub trait GameTreeVertex
where
    // Weird problem here which requires that GameTreeVertex be both Clone + Ord when
    // we only require `GameTreeVertex::PlayerInfo` and `GameTreeVertex::Action` to
    // derive those traits.
    Self: Debug + Clone,
{
    // Information set for a given state (assuming its a player's move).
    type PlayerInfo: Eq + Ord + Debug + Clone;

    // Actions which could be taken. These could be either from a player's move or chance.
    // Actions between information sets *may* be the same, so this should not be
    // used as a sequence identifier! To do so, use a (PlayerInfo, Aciton) tuple instead,
    // see the `Sequence` struct later on.
    type Action: Eq + Ord + Debug + Clone;

    // Description of a subgame. Associated types are not allowed in stable rust
    // (as of June '19). If there are no subgames, one still has to define a dummy
    // type in the implementation, e.g.,
    // type SubGame = usize;
    type Subgame: Eq + Ord + Debug + Clone; // = usize;

    fn next_player(&self) -> ChanceOrPlayer;
    fn player_information(&self) -> Self::PlayerInfo;
    fn available_actions(&self) -> Box<[(Self::Action, f64)]>;
    fn next_state(&self, action: &Self::Action) -> VertexOrLeaf<Self>;

    /// Return subgame that the vertex belongs to, and None if the vertex is not
    /// in a subgame. Returns `None` by default, meaning that the calling vertex is
    /// not in any subgame.
    fn subgame(&self) -> Option<Self::Subgame> {
        None
    }

    /// Used for sanity checks during tree traversals. Can possibly panic when
    /// we reach an impossible state. This function is only for debugging and
    /// is required to be implemented.
    fn validate(&self) {}
}

/// Indicates if the node is a chance node or a player's decision point.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ChanceOrPlayer {
    Player(Player),
    Chance,
}

/// Indicates if we are at a vertex (chance or player) or a leaf (terminal) vertex.
pub enum VertexOrLeaf<T: GameTreeVertex> {
    Leaf(Leaf),
    Vertex(T),
}

/// Contains details of leaf (terminal nodes) in the original game tree.
/// We require that the game generator outputs a `Leaf` object at terminal
/// states of the game.
/// Note that `Leaf` should not include chance factors or sequences preceding
/// it, those will be automatically computed by this library --- specifically,
/// this is not equal to the leaf in a treeplex.
#[derive(Copy, Clone, Debug)]
pub struct Leaf {
    pub payoff_pl1: f64,
    pub payoff_pl2: f64,
}