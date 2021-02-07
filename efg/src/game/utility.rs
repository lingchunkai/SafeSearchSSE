use crate::game::ExtensiveFormGame;
use crate::game::PayoffMatrixEntry;
use crate::game::Player;
use crate::treeplex::SequenceId;
use crate::treeplex::TreeplexTools;

#[derive(Debug)]
pub struct EFGTools {
    num_payoff_entries: usize,
    //
    // `payoffs_sorted_plx`
    // Indices of game.payoff_matrix, sorted in increasing order of player 1's SequenceId.
    // `payoffs_range_sorted_plx`
    // Index range represented by [start, end] for each terminal sequence.
    //
    // Example usage
    // =============
    // Suppose that we need to find all payoff_entries between sequence i and j (EXCLUSIVE)
    // of player 1---for example, if sequences [i, j) are the sequences in a subtree,
    // i.e., a subgame.
    //
    // Then, let
    //      l = payoffs_range_sorted_pl1[i].0 and
    //      r = payoffs_range_sorted_pl1[j].1.
    // The indices of the payoff_matrix are game.payoff_matrix.entries[x] for
    //      x = payoffs_sorted_pl1[k] for k in [l, r).
    //
    payoffs_sorted_pl1: Vec<usize>,
    payoffs_sorted_pl2: Vec<usize>,
    payoffs_range_sorted_pl1: Vec<(usize, usize)>,
    payoffs_range_sorted_pl2: Vec<(usize, usize)>,
}

impl EFGTools {
    pub fn new(game: &ExtensiveFormGame) -> EFGTools {
        let mut efg_tools = EFGTools {
            num_payoff_entries: game.num_payoff_entries(),
            payoffs_sorted_pl1: Vec::<_>::new(),
            payoffs_sorted_pl2: Vec::<_>::new(),
            payoffs_range_sorted_pl1: Vec::<_>::new(),
            payoffs_range_sorted_pl2: Vec::<_>::new(),
        };
        efg_tools.precompute_payoffs_sorted(game, Player::Player1);
        efg_tools.precompute_payoffs_sorted(game, Player::Player2);
        efg_tools.precompute_payoffs_range_sorted_pl1(game);
        efg_tools.precompute_payoffs_range_sorted_pl2(game);

        efg_tools
    }

    /// Returns a vector of of payoff matrix indices for each leaf that leaves at or a
    /// particular information set.
    /// TODO (chunkail): change the return value from vectors to an iterator of `usize`.
    pub fn leaf_indices_at_or_under_infoset(
        &self,
        player: Player,
        treeplex_tools: &TreeplexTools,
        infoset_id: usize,
    ) -> Vec<usize> {
        match player {
            Player::Player1 => {
                let (start_sequence, end_sequence) = treeplex_tools.seqs_under_infoset(infoset_id);
                let l = self.payoffs_range_sorted_pl1[start_sequence].0;
                let r = self.payoffs_range_sorted_pl1[end_sequence].1;
                (l..r).map(|x| self.payoffs_sorted_pl1[x]).collect()
            }
            Player::Player2 => {
                let (start_sequence, end_sequence) = treeplex_tools.seqs_under_infoset(infoset_id);
                let l = self.payoffs_range_sorted_pl2[start_sequence].0;
                let r = self.payoffs_range_sorted_pl2[end_sequence].1;
                (l..r).map(|x| self.payoffs_sorted_pl2[x]).collect()
            }
        }
    }

    /// 
    pub fn leaf_indices_at_sequence(
        &self,
        player: Player,
        sequence_id: SequenceId,
    ) -> Vec<usize> {
        match player {
            Player::Player1 => {
                let l = self.payoffs_range_sorted_pl1[sequence_id].0;
                let r = self.payoffs_range_sorted_pl1[sequence_id].1;
                (l..r).map(|x| self.payoffs_sorted_pl1[x]).collect()
            },
            Player::Player2 => {
                let l = self.payoffs_range_sorted_pl2[sequence_id].0;
                let r = self.payoffs_range_sorted_pl2[sequence_id].1;
                (l..r).map(|x| self.payoffs_sorted_pl2[x]).collect()
            }
        }
    }

    ///
    fn precompute_payoffs_sorted(&mut self, game: &ExtensiveFormGame, player: Player) {
        match player {
            Player::Player1 => {
                assert!(
                    self.payoffs_sorted_pl1.len() == 0,
                    "payoffs-sorted-pl1 is non-empty"
                );
                let mut indices: Vec<_> = (0..game.num_payoff_entries()).collect();
                indices.sort_by_key(|x| game.payoff_entry(*x).seq_pl1);
                self.payoffs_sorted_pl1 = indices;
            }
            Player::Player2 => {
                assert!(
                    self.payoffs_sorted_pl2.len() == 0,
                    "payoffs-sorted-pl2 is non-empty"
                );
                let mut indices: Vec<_> = (0..game.num_payoff_entries()).collect();
                indices.sort_by_key(|x| game.payoff_entry(*x).seq_pl2);
                self.payoffs_sorted_pl2 = indices;
            }
        }
    }

    fn precompute_payoffs_range_sorted_pl1(&mut self, game: &ExtensiveFormGame) {
        // TODO (chunkail): refactor using iterators, using 'take-while'.
        let treeplex = game.treeplex(Player::Player1);
        assert!(
            self.payoffs_range_sorted_pl1.len() == 0,
            "payoffs-range-sorted-pl1 is non-empty"
        );

        let mut cur_payoff_entry_sorted_id: usize = 0;
        for sequence_id in 0..treeplex.num_sequences() {
            // Default start and end is an `empty` range starting at `cur_payoff_entry_sorted_id`.
            let (start_payoff_entry_sorted, mut end_payoff_entry_sorted) =
                (cur_payoff_entry_sorted_id, cur_payoff_entry_sorted_id);

            // Take as many payoff entries as possible.
            while cur_payoff_entry_sorted_id < game.num_payoff_entries() {
                // Payoff entry id in the payoff matrix.
                let cur_payoff_entry_id = self.payoffs_sorted_pl1[cur_payoff_entry_sorted_id];
                let payoff_entry = game.payoff_entry(cur_payoff_entry_id);

                assert!(
                    payoff_entry.seq_pl1 >= sequence_id,
                    "Sorting error encountered! `payoff-entry-id` is not sorted properly."
                );

                // We have reached the end of this block (if any)
                if payoff_entry.seq_pl1 > sequence_id {
                    break;
                }

                cur_payoff_entry_sorted_id += 1;
                end_payoff_entry_sorted += 1;
            }
            self.payoffs_range_sorted_pl1
                .push((start_payoff_entry_sorted, end_payoff_entry_sorted));
        }
        assert!(self.payoffs_range_sorted_pl1.len() == treeplex.num_sequences());
    }

    fn precompute_payoffs_range_sorted_pl2(&mut self, game: &ExtensiveFormGame) {
        // TODO (chunkail): refactor using iterators, using 'take-while'.
        let treeplex = game.treeplex(Player::Player2);
        assert!(
            self.payoffs_range_sorted_pl2.len() == 0,
            "payoffs-range-sorted-pl1 is non-empty"
        );

        let mut cur_payoff_entry_sorted_id: usize = 0;
        for sequence_id in 0..treeplex.num_sequences() {
            // Default start and end is an `empty` range starting at `cur_payoff_entry_sorted_id`.
            let (start_payoff_entry_sorted, mut end_payoff_entry_sorted) =
                (cur_payoff_entry_sorted_id, cur_payoff_entry_sorted_id);

            // Take as many payoff entries as possible.
            while cur_payoff_entry_sorted_id < game.num_payoff_entries() {
                // Payoff entry id in the payoff matrix.
                let cur_payoff_entry_id = self.payoffs_sorted_pl2[cur_payoff_entry_sorted_id];
                let payoff_entry = game.payoff_entry(cur_payoff_entry_id);

                assert!(
                    payoff_entry.seq_pl2 >= sequence_id,
                    "Sorting error encountered! `payoff-entry-id` is not sorted properly."
                );

                // We have reached the end of this block (if any)
                if payoff_entry.seq_pl2 > sequence_id {
                    break;
                }

                cur_payoff_entry_sorted_id += 1;
                end_payoff_entry_sorted += 1;
            }
            self.payoffs_range_sorted_pl2
                .push((start_payoff_entry_sorted, end_payoff_entry_sorted));
        }
        assert!(self.payoffs_range_sorted_pl2.len() == treeplex.num_sequences());
    }
}