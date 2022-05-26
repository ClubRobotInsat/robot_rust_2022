use crate::protocol::errors::ProtocolError;
use crate::protocol::{CanId, MessageId, SeqId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub(crate) id_dest: CanId,        // really a u4
    pub(crate) id_src: CanId,         // really a u4
    pub(crate) is_ack: bool,          // really a u1
    pub(crate) id_message: MessageId, // really a u3
    pub(crate) seq_number: SeqId,     // really a u4
}

impl Header {
    const DEST_MASK: u8 = 0b11110000;
    const DEST_OFFSET: usize = 4;

    const SRC_MASK: u8 = 0b00001111;
    const SRC_OFFSET: usize = 0;

    const ID_MESSAGE_MASK: u8 = 0b11100000;
    const ID_MESSAGE_OFFSET: usize = 5;

    const SEQ_NUMBER_MASK: u8 = 0b00011110;
    const SEQ_NUMBER_OFFSET: usize = 1;

    const IS_ACK_MASK: u8 = 0b00000001;
    const IS_ACK_OFFSET: usize = 0;

    /// Creates a new header with the given parameters
    /// # Warnining
    /// take care of the size of the parameters, the function will return an error if the size is not correct
    /// # Arguments
    /// id_dest: u4,
    /// id_src: u4,
    /// is_ack: u1,
    /// id_message: u3,
    /// seq_number: u4
    pub fn new(
        id_dest: CanId,
        id_src: CanId,
        is_ack: bool,
        id_message: MessageId,
        seq_number: SeqId,
    ) -> Result<Header, ProtocolError> {
        if id_dest == id_src {
            return Err(ProtocolError::SrcAndDestCanNotBeEqual);
        }

        Ok(Header {
            id_dest,
            id_src,
            is_ack,
            id_message,
            seq_number,
        })
    }

    /// Returns the correspondant header from a message
    /// # Warning
    /// If the message received was not well formatted the result of this function might be
    /// erroneous as there is no way to know it=
    pub fn new_from_binary_array(array: &[u8; 2]) -> Result<Header, ProtocolError> {
        let id_dest_raw = (array[0] & Self::DEST_MASK) >> Self::DEST_OFFSET;
        let id_src_raw = (array[0] & Self::SRC_MASK) >> Self::SRC_OFFSET;
        let id_message_raw = (array[1] & Self::ID_MESSAGE_MASK) >> Self::ID_MESSAGE_OFFSET;
        let seq_number_raw = (array[1] & Self::SEQ_NUMBER_MASK) >> Self::SEQ_NUMBER_OFFSET;
        let is_ack_raw = ((array[1] & Self::IS_ACK_MASK) >> Self::IS_ACK_OFFSET);

        Ok(Header {
            // Can't panic because CanId is u4
            id_dest: CanId::new(id_dest_raw as usize).unwrap(),
            // Can't panic because CanId is u4
            id_src: CanId::new(id_src_raw as usize).unwrap(),
            // Can't panic because CanId is u4
            is_ack: is_ack_raw == 1,
            // Can't panic because MessageId is u4
            id_message: MessageId::new(id_message_raw as usize).unwrap(),
            // Can't panic because SeqId is u4
            seq_number: SeqId::new(seq_number_raw as usize).unwrap(),
        })
    }
    pub fn get_id_dest(&self) -> CanId {
        self.id_dest
    }
    pub fn get_id_src(&self) -> CanId {
        self.id_src
    }
    pub fn get_is_ack(&self) -> bool {
        self.is_ack
    }
    pub fn get_id_message(&self) -> MessageId {
        self.id_message
    }
    pub fn get_seq_number(&self) -> SeqId {
        self.seq_number
    }
}
