use crate::treeplex::SequenceId;

use crate::schema::game_capnp;

#[derive(Debug, Copy, Clone)]
pub struct Infoset {
    pub parent_sequence: SequenceId,
    pub start_sequence: SequenceId,
    pub end_sequence: SequenceId,
}

impl Infoset {
    pub fn new(
        parent_sequence: SequenceId,
        start_sequence: SequenceId,
        end_sequence: SequenceId,
    ) -> Infoset {
        Infoset {
            parent_sequence,
            start_sequence,
            end_sequence,
        }
    }

    /// Serializs the infoset as a Cap'n'proto structure.
    pub fn serialize<'b>(&self, builder: &mut game_capnp::infoset::Builder<'b>) {
        builder.set_start_sequence_id(self.start_sequence as u32);
        builder.set_end_sequence_id(self.end_sequence as u32);
        builder.set_parent_sequence_id(self.parent_sequence as u32);
    }

    /// Deserializs an infoset from a Cap'n'proto structure.
    pub fn deserialize<'b>(reader: &game_capnp::infoset::Reader<'b>) -> Infoset {
        Infoset::new(
            reader.get_parent_sequence_id() as SequenceId,
            reader.get_start_sequence_id() as SequenceId,
            reader.get_end_sequence_id() as SequenceId,
        )
    }
}