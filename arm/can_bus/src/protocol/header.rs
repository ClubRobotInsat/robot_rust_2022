use crate::protocol::errors::ProtocolError;
use crate::protocol::{CanId, MessageId, SeqId};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub id_dest: CanId,
    pub id_src: CanId,
    pub is_ack: bool,
    pub id_message: MessageId,
    pub seq_number: SeqId,
}

impl Header {
    /// Creates a new header with the given parameters
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
}

impl TryFrom<&[u8; 8]> for Header {
    type Error = ProtocolError;

    fn try_from(array: &[u8; 8]) -> Result<Self, Self::Error> {
        let id_dest_raw = (array[0] & DEST_MASK) >> DEST_OFFSET;
        let id_src_raw = (array[0] & SRC_MASK) >> SRC_OFFSET;
        let id_message_raw = (array[1] & ID_MESSAGE_MASK) >> ID_MESSAGE_OFFSET;
        let seq_number_raw = (array[1] & SEQ_NUMBER_MASK) >> SEQ_NUMBER_OFFSET;
        let is_ack_raw = (array[1] & IS_ACK_MASK) >> IS_ACK_OFFSET;

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
}

impl TryInto<[u8; 2]> for &Header {
    type Error = ProtocolError;

    fn try_into(self) -> Result<[u8; 2], Self::Error> {
        let mut raw = [0u8, 0u8];

        raw[0] |= (self.id_dest.v << DEST_OFFSET) as u8;
        raw[0] |= (self.id_src.v << SRC_OFFSET) as u8;
        raw[1] |= (self.id_message.v << ID_MESSAGE_OFFSET) as u8;
        raw[1] |= (self.seq_number.v << SEQ_NUMBER_OFFSET) as u8;
        raw[1] |= (if self.is_ack { 1 } else { 0 } << IS_ACK_OFFSET) as u8;

        Ok(raw)
    }
}
