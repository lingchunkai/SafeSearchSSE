use crate::game::Player;
use crate::game::{PayoffMatrix, PayoffMatrixEntry};
use crate::strategy::SequenceFormStrategy;

use crate::treeplex::Treeplex;
use crate::vector::TreeplexVector;

use crate::schema::game_capnp;
use capnp;
use std::rc::Rc;
/// This library is a lite version of libefg, adapted from Gabriele Farina (gfarina@cs.cmu.edu).
/// We have taken the liberty to modify the code such that the code is tailored towards
/// the extensive-form case with some support for subgame-solving.
///
/// For 2-player Stackelberg settings, we will use the convention that pl1 is the leader and
/// pl2 is the follower. This is to allow for future extensions for other 2-player
/// equilibrium concepts beyond Stackelberg equliibria.
///
/// Currently the code only supports games non-nested subgames.
///
/// The versions of `ExtensiveFormGame` are similar to Gabriele Farina and is written such that
/// if one does not specify any subgames, then the two structs are in fact, functionally identical.
/// TODO(chunkail) -- how about subclassing from Gabriele's libefg?
///
/// `ExtensiveFormGame` and `Treeplex` both have the same requirements as in libefg.
/// However, in order to facilitate subgame-solving, we also require that each subgame has leaves
/// which are continguous in the payoff_matrix entries. It is crucial that game generation takes
/// this into account, either by specifying some index for subgames when enumerating leaf indices,
/// followed by a round of sorting, or by performing game generation in a special order.
#[derive(Debug, Clone)]
pub struct ExtensiveFormGame {
    // Full game treeplexes
    treeplex_pl1: Rc<Treeplex>,
    treeplex_pl2: Rc<Treeplex>,

    // Sparse sequence-form payoff matrix
    payoff_matrix: PayoffMatrix,

    // Subgame indices for each information set
    subgames_pl1: Vec<SubgameOrFree>,
    subgames_pl2: Vec<SubgameOrFree>,

    num_subgames: usize,
}

/// Specifies if an object within any subgame, i.e., "free", or belonging to a subgame
/// with index given in ::Subgame.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub enum SubgameOrFree {
    Free,
    Subgame(usize),
}

impl SubgameOrFree {
    pub fn to_integer(&self) -> usize {
        match self {
            SubgameOrFree::Free => 0,
            SubgameOrFree::Subgame(x) => x + 1,
        }
    }

    pub fn from_integer(idx: usize) -> SubgameOrFree {
        if idx == 0 {
            SubgameOrFree::Free
        } else {
            SubgameOrFree::Subgame(idx - 1)
        }
    }
}

impl ExtensiveFormGame {
    pub fn new(
        treeplex_pl1: Rc<Treeplex>,
        treeplex_pl2: Rc<Treeplex>,
        payoff_matrix: PayoffMatrix,
        subgames_pl1: Vec<SubgameOrFree>,
        subgames_pl2: Vec<SubgameOrFree>,
    ) -> ExtensiveFormGame {
        let num_subgames = std::cmp::max(
            Self::compute_num_subgames(&subgames_pl1),
            Self::compute_num_subgames(&subgames_pl2),
        );

        ExtensiveFormGame {
            treeplex_pl1,
            treeplex_pl2,
            payoff_matrix,
            subgames_pl1,
            subgames_pl2,
            num_subgames,
        }
    }

    pub fn evaluate_payoffs(
        &self,
        seq_pl1: &SequenceFormStrategy,
        seq_pl2: &SequenceFormStrategy,
        player: Player,
    ) -> f64 {
        match player {
            Player::Player1 => { 
                let gradient = self.gradient(Player::Player1, seq_pl2);
                gradient.inner(seq_pl1.inner())
            }
            Player::Player2 => { 
                let gradient = self.gradient(Player::Player2, seq_pl1);
                gradient.inner(seq_pl2.inner())
            }
        }
    }

    /// Compute the gradient of the payoffs with respect to `player`.
    /// That is, suppose x_i, \in {1, 2} are sequence form vectors, and
    /// P_i are payoff matrices for player i, such that
    ///                x_1 P_1 x_2 and
    ///                x_1 P_2 x_2.
    /// are payoffs for Player 1 and Player 2 respectively. The gradient
    /// for Player 1 and 2 are
    ///                P_1 x_2 and P_2^T x_1
    /// respectively. Note that the payoff matrix of the i-th player i used.
    /// It should also be noted that the sequence-form strategy
    /// which is provided belongs to player -i, and *not* i.
    ///
    /// This method is typically used as a step towards computing the
    /// best-response to a particular player's sequence form strategy.
    pub fn gradient(
        &self,
        player: Player,
        sequence_form_strategy: &SequenceFormStrategy,
    ) -> TreeplexVector {
        self.gradient_for_payoffs(player, player, sequence_form_strategy)
    }

    /// Same as `gradient()` but with a more fine-grained parameters.
    /// When taking gradients of Player i (i.e., with respect to x_i),
    /// we do not assume that the payoffs we are referring to are
    /// for Player i as well; these could be for Player -i. For example,
    /// we could compute
    ///                P_2 x_2 and
    ///                P_1 x_1
    /// as well. These quantities are useful in several settings, most notably
    /// that of Stackelberg games, where tiebreaking by the attacker
    /// is performed in favour of the defender.
    ///
    /// The target_player parameter refers to the Player i, i.e., the index
    /// of x that we are taking gradients of. The payoff_player parameter
    /// is j, i.e., the payoff values tha we are using.
    pub fn gradient_for_payoffs(
        &self,
        target_player: Player,
        payoff_player: Player,
        sequence_form_strategy: &SequenceFormStrategy,
    ) -> TreeplexVector {
        // Ensure that we are taking gradients of the player whose sequence form strategy is not given.
        assert_eq!(
            target_player,
            -sequence_form_strategy.inner().treeplex().player()
        );

        let mut gradient = TreeplexVector::from_constant(self.treeplex(target_player), 0f64);

        // Enumerate all 4 permutations and handle each separately.
        match (target_player, payoff_player) {
            (Player::Player1, Player::Player1) => {
                for p in self.payoff_matrix.entries.iter() {
                    gradient[p.seq_pl1] +=
                        p.chance_factor * sequence_form_strategy.inner()[p.seq_pl2] * p.payoff_pl1;
                }
            }
            (Player::Player2, Player::Player2) => {
                for p in self.payoff_matrix.entries.iter() {
                    gradient[p.seq_pl2] +=
                        p.chance_factor * sequence_form_strategy.inner()[p.seq_pl1] * p.payoff_pl2;
                }
            }
            (Player::Player1, Player::Player2) => {
                for p in self.payoff_matrix.entries.iter() {
                    gradient[p.seq_pl1] +=
                        p.chance_factor * sequence_form_strategy.inner()[p.seq_pl2] * p.payoff_pl2;
                }
            }
            (Player::Player2, Player::Player1) => {
                for p in self.payoff_matrix.entries.iter() {
                    gradient[p.seq_pl2] +=
                        p.chance_factor * sequence_form_strategy.inner()[p.seq_pl1] * p.payoff_pl1;
                }
            }
        }
        gradient
    }

    pub fn zero_sum(&self, player: Player) -> ExtensiveFormGame {
        let mut efg = self.clone();
        let new_payoff_matrix = PayoffMatrix::new(efg.payoff_matrix().entries.iter().map(|&x| {
                let mut y = x.clone();
                match player {
                    Player::Player1 => y.payoff_pl2 = -y.payoff_pl1,
                    Player::Player2 => y.payoff_pl1 = -y.payoff_pl2,
                };
                y
        }).collect::<Vec<PayoffMatrixEntry>>());

        efg.payoff_matrix = new_payoff_matrix;
        
        efg
    }

    pub fn is_zero_sum(&self) -> bool {
        self.payoff_matrix().entries.iter().all(|&x| ulps_eq!(x.payoff_pl1, -x.payoff_pl2))
    }

    pub fn treeplex(&self, player: Player) -> &Treeplex {
        match player {
            Player::Player1 => &self.treeplex_pl1,
            Player::Player2 => &self.treeplex_pl2,
        }
    }

    pub fn payoff_matrix(&self) -> &PayoffMatrix {
        &self.payoff_matrix
    }

    pub fn payoff_entry(&self, payoff_index: usize) -> PayoffMatrixEntry {
        self.payoff_matrix.entries[payoff_index]
    }

    pub fn num_payoff_entries(&self) -> usize {
        self.payoff_matrix.entries.len()
    }

    pub fn subgame(&self, player: Player, infoset_id: usize) -> SubgameOrFree {
        match player {
            Player::Player1 => self.subgames_pl1[infoset_id],
            Player::Player2 => self.subgames_pl2[infoset_id],
        }
    }

    pub fn num_subgames(&self) -> usize {
        self.num_subgames
    }


    pub fn serialize<'b>(&self, builder: &mut game_capnp::game::Builder<'b>) {
        let mut treeplex_pl1_builder = builder.reborrow().init_treeplex_pl1();
        self.treeplex(Player::Player1)
            .serialize(&mut treeplex_pl1_builder);

        let mut treeplex_pl2_builder = builder.reborrow().init_treeplex_pl2();
        self.treeplex(Player::Player2)
            .serialize(&mut treeplex_pl2_builder);

        let mut payoff_matrix_builder = builder.reborrow().init_payoff_matrix();
        self.payoff_matrix.serialize(&mut payoff_matrix_builder);

        let mut subgames_pl1_builder = builder
            .reborrow()
            .init_subgames_pl1(self.treeplex_pl1.num_infosets() as u32);
        for (infoset_index, subgame) in self.subgames_pl1.iter().enumerate() {
            subgames_pl1_builder.set(infoset_index as u32, subgame.to_integer() as u32);
        }

        let mut subgames_pl2_builder = builder
            .reborrow()
            .init_subgames_pl2(self.treeplex_pl2.num_infosets() as u32);
        for (infoset_index, subgame) in self.subgames_pl2.iter().enumerate() {
            subgames_pl2_builder.set(infoset_index as u32, subgame.to_integer() as u32);
        }
    }

    pub fn deserialize(game_reader: &game_capnp::game::Reader) -> capnp::Result<ExtensiveFormGame> {
        let treeplex_pl1 = Rc::new(Treeplex::deserialize(
            Player::Player1,
            &game_reader.get_treeplex_pl1()?,
        )?);
        let treeplex_pl2 = Rc::new(Treeplex::deserialize(
            Player::Player2,
            &game_reader.get_treeplex_pl2()?,
        )?);
        let payoff_matrix = PayoffMatrix::deserialize(&game_reader.get_payoff_matrix()?)?;

        let mut subgames_pl1 = vec![];
        for subgame in game_reader.get_subgames_pl1()?.iter() {
            subgames_pl1.push(SubgameOrFree::from_integer(subgame as usize));
        }

        let mut subgames_pl2 = vec![];
        for subgame in game_reader.get_subgames_pl2()?.iter() {
            subgames_pl2.push(SubgameOrFree::from_integer(subgame as usize));
        }

        Ok(ExtensiveFormGame::new(
            treeplex_pl1,
            treeplex_pl2,
            payoff_matrix,
            subgames_pl1,
            subgames_pl2,
        ))
    }

    pub fn persist<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut message_builder = capnp::message::Builder::new_default();

        let mut game_builder = message_builder.init_root::<game_capnp::game::Builder>();
        self.serialize(&mut game_builder);

        capnp::serialize::write_message(writer, &message_builder)
    }

    fn compute_num_subgames(subgames: &Vec<SubgameOrFree>) -> usize {
        subgames.iter().fold(0, |accum, x| {
            std::cmp::max(
                accum,
                match x {
                    SubgameOrFree::Free => 0,
                    SubgameOrFree::Subgame(y) => y + 1,
                },
            )
        })
    }
}

#[cfg(test)]
pub mod test_fixtures {
    use crate::treeplex::Treeplex;

    // TODO(chunkail): Learn how to import test fixtures from other modules...
    //
    // For Player 1, there are 4 terminal seqences (multiplied by 3 for the card that he obtained)
    // (A) Check [Other player checks as well]
    // (B) Check, Raise, Fold
    // (C) Check, Raise, Raise
    // (D) Raise [Other player amy choose to raise or fold]
    //
    // For Player 2, there are 4 terminal sequences (multiplied by 3 as usual)
    // (1) Check, Check
    // (2) Check, Raise
    // (3) Raise, Fold
    // (4) Raise, Raise
    //
    // Compatible sequences are quite rare and may be enumerated. They are given by
    // (Check, Check)
    // (Raise, )

    /*
    pub static ref KUHN_PAYOFFS: PayoffMatrix = { entries:
        // vec![PayoffMatrixEntry(0, )
        };
    */
}