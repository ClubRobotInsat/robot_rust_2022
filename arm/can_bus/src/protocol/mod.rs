//

pub mod errors;
pub mod header;
pub mod message;
pub mod message_in_progress;
pub mod protocol;
pub mod packet;

use crate::protocol::errors::ProtocolError;

pub const U4_MAX: usize = 2usize.pow(4) - 1;
pub const U3_MAX: usize = 2usize.pow(3) - 1;

pub const MAX_SEQ_NUMBER: usize = U4_MAX;
pub const PACKET_DATA_SIZE: usize = 6;
pub const CAN_PACKET_SIZE: usize = 8;
pub const MAX_MESSAGE_LEN: usize = MAX_SEQ_NUMBER * 6;

pub const MAX_CAN_ID: usize = U4_MAX;
pub const MAX_MES_ID: usize = U3_MAX;
pub const MAX_SEQ_ID: usize = U4_MAX;

macro_rules! can_id {
    ($name:ident, $max_size:expr) => {
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
        pub struct $name {
            v: usize,
        }

        impl $name {
            pub fn new(v: usize) -> Result<Self, ProtocolError> {
                if v > $max_size {
                    Err(ProtocolError::InvalidId(v))
                } else {
                    Ok(Self { v })
                }
            }
        }

        impl From<$name> for usize {
            fn from(s: $name) -> Self {
                s.v as usize
            }
        }
    };
}

can_id!(CanId, MAX_CAN_ID);
can_id!(MessageId, MAX_MES_ID);

// SeqId(0) is the last packet
can_id!(SeqId, MAX_SEQ_ID);
