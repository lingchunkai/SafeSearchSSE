use crate::treeplex::SequenceId;
use crate::schema::game_capnp;

use std::collections::BTreeMap;
use capnp;

#[derive(Debug, Clone)]
pub struct PayoffMatrix {
    pub entries: Vec<PayoffMatrixEntry>,
}

#[derive(Debug, Copy, Clone)]
pub struct PayoffMatrixEntry {
    pub seq_pl1: SequenceId,
    pub seq_pl2: SequenceId,
    pub chance_factor: f64,
    pub payoff_pl1: f64,
    pub payoff_pl2: f64,
}

impl PayoffMatrix {
    pub fn new(entries: Vec<PayoffMatrixEntry>) -> PayoffMatrix {
        let mut payoff_matrix = PayoffMatrix { entries: entries };
        payoff_matrix.flatten();
        payoff_matrix
    }

    /// Sometimes, there may be cases when there are two payoff entries 
    /// which share the exact same *pair* of sequences. For example, in Leduc poker,
    /// if the game tree is generated top down (with the public, unrevealed card)
    /// "drawn" before being revealed, then there may be two leaves in the game tree
    /// that are actually same entry in the sequence form payoff matrix.
    /// The flatten() function combines these duplicate entries into a single entry.
    pub fn flatten(&mut self) {
        let mut map = BTreeMap::<(SequenceId, SequenceId), PayoffMatrixEntry>::new();
        
        for payoff_entry in self.entries.iter() {
            let (seq_pl1, seq_pl2) = (payoff_entry.seq_pl1, payoff_entry.seq_pl2);
            if !map.contains_key(&(seq_pl1, seq_pl2)) {
                map.insert((seq_pl1, seq_pl2), payoff_entry.clone());
            } else {
                let old_payoff_entry = map.get(&(seq_pl1, seq_pl2)).unwrap();
                let new_chance_factor = old_payoff_entry.chance_factor + payoff_entry.chance_factor;
                let expected_util_pl1 = payoff_entry.payoff_pl1 * payoff_entry.chance_factor 
                                        + old_payoff_entry.payoff_pl1 * old_payoff_entry.chance_factor;
                let expected_util_pl2 = payoff_entry.payoff_pl2 * payoff_entry.chance_factor
                                        + old_payoff_entry.payoff_pl2 * old_payoff_entry.chance_factor;
                if new_chance_factor <= std::f64::EPSILON {
                    continue;
                }
                map.insert((seq_pl1, seq_pl2), 
                            PayoffMatrixEntry {
                                seq_pl1, 
                                seq_pl2,
                                chance_factor: new_chance_factor,
                                payoff_pl1: expected_util_pl1 / new_chance_factor,
                                payoff_pl2: expected_util_pl2 / new_chance_factor,
                            });
            }

        }

        self.entries = map.values().cloned().collect::<Vec<PayoffMatrixEntry>>();
    }

    pub fn serialize<'b>(&self, builder: &mut game_capnp::payoff_matrix::Builder<'b>) {
        let mut entries_builder = builder.reborrow().init_entries(self.entries.len() as u32);

        for (entry_index, payoff_matrix_entry) in self.entries.iter().enumerate() {
            let mut payoff_matrix_entry_builder =
                entries_builder.reborrow().get(entry_index as u32);
            payoff_matrix_entry.serialize(&mut payoff_matrix_entry_builder);
        }
    }

    pub fn deserialize<'b>(
        reader: &game_capnp::payoff_matrix::Reader<'b>,
    ) -> capnp::Result<PayoffMatrix> {
        let mut entries = vec![];
        for payoff_matrix_entry in reader.get_entries()?.iter() {
            entries.push(PayoffMatrixEntry::deserialize(&payoff_matrix_entry));
        }
        Ok(PayoffMatrix::new(entries))
    }
}

impl PayoffMatrixEntry {
    pub fn new(
        seq_pl1: SequenceId,
        seq_pl2: SequenceId,
        chance_factor: f64,
        payoff_pl1: f64,
        payoff_pl2: f64,
    ) -> PayoffMatrixEntry {
        PayoffMatrixEntry {
            seq_pl1,
            seq_pl2,
            chance_factor,
            payoff_pl1,
            payoff_pl2,
        }
    }

    pub fn serialize<'b>(
        &self,
        builder: &mut game_capnp::payoff_matrix::payoff_matrix_entry::Builder<'b>,
    ) {
        builder.set_seq_pl1(self.seq_pl1 as u32);
        builder.set_seq_pl2(self.seq_pl2 as u32);
        builder.set_chance_factor(self.chance_factor);
        builder.set_payoff_pl1(self.payoff_pl1);
        builder.set_payoff_pl2(self.payoff_pl2);
    }

    pub fn deserialize<'b>(
        reader: &game_capnp::payoff_matrix::payoff_matrix_entry::Reader<'b>,
    ) -> PayoffMatrixEntry {
        PayoffMatrixEntry::new(
            reader.get_seq_pl1() as SequenceId,
            reader.get_seq_pl2() as SequenceId,
            reader.get_chance_factor() as f64,
            reader.get_payoff_pl1() as f64,
            reader.get_payoff_pl2() as f64,
        )
    }
}