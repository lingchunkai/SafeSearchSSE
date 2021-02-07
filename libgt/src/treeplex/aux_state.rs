use crate::treeplex::{Sequence, SequenceOrEmpty};
use crate::efg_lite::game::{Player, SubgameOrFree};

/// `AuxState` records a succinct summary of path taken in the game
/// tree traversal, including the player sequences traversed, cumulative
/// probabilities required by the chance player, and previous subgame (index)
/// for each player.
#[derive(Debug, Clone, Copy)]
pub struct AuxState {
    /// Last sequence that Player 1 took prior to this state.
    pub prev_seq_pl1: SequenceOrEmpty,

    /// Last sequence that Player 2 took prior to this state.
    pub prev_seq_pl2: SequenceOrEmpty,

    /// Product of chance probabilities taken before reaching this state.
    pub chance_factor: f64,

    /// Last subgame (or empty) that was encountered prior to this state.
    pub prev_subgame: SubgameOrFree,
}

impl AuxState {
    /// Create a new auxillary state with only the chance factor and subgame modified.
    pub fn new_with_updated_chance(&self, chance_to_multiply: f64, new_subgame: SubgameOrFree) -> AuxState {
        AuxState {
            prev_seq_pl1: self.prev_seq_pl1,
            prev_seq_pl2: self.prev_seq_pl2,
            chance_factor: chance_to_multiply * self.chance_factor,
            prev_subgame: new_subgame,
        }
    }

    /// Create a new auxillary state with exactly one of the player's sequence, 
    /// as well as the subgame modified.
    pub fn new_with_updated_sequence(&self, player: Player, new_seq: Sequence, new_subgame: SubgameOrFree) -> AuxState {
        match player {
            Player::Player1 => AuxState {
                prev_seq_pl1: SequenceOrEmpty::Sequence(new_seq),
                prev_seq_pl2: self.prev_seq_pl2,
                chance_factor: self.chance_factor,
                prev_subgame: new_subgame,
            },
            Player::Player2 => AuxState {
                prev_seq_pl1: self.prev_seq_pl1,
                prev_seq_pl2: SequenceOrEmpty::Sequence(new_seq),
                chance_factor: self.chance_factor,
                prev_subgame: new_subgame,
            },
        }
    }
}