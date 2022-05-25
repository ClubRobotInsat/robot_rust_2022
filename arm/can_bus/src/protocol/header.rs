use crate::protocol::errors::MessageCreationError;
use crate::protocol::{MAX_ID, MAX_ID_MESS, MAX_SEQ_NUMBER};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    id_dest: u8,    // really a u4
    id_src: u8,     // really a u4
    is_ack: bool,   // really a u1
    id_message: u8, // really a u3
    seq_number: u8, // really a u4
}

impl Header {
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
        id_dest: u8,
        id_src: u8,
        is_ack: bool,
        id_message: u8,
        seq_number: u8,
    ) -> Result<Header, MessageCreationError> {
        if id_dest > MAX_ID
            || id_src > MAX_ID
            || id_message > MAX_ID_MESS
            || seq_number > MAX_SEQ_NUMBER
        {
            return Err(MessageCreationError::ParametersTooLong);
        }
        if id_dest == id_src {
            return Err(MessageCreationError::SrcAndDestCanNotBeEqual);
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
    /// If the message received was not well formatted the result of this function might be erroneous as there is no way to know it
    pub fn new_from_binary_array(array: &[u8; 2]) -> Header {
        let id_dest: u8 = array[0] >> 4; // first 4 bits
        let id_src: u8 = array[0] & 0x0F; // last 4 bits
        let id_message: u8 = array[1] >> 5; // first 3 bits
        let seq_number: u8 = (array[1] & 0b00011110) >> 1; //  4 bits
        let is_ack: bool = (array[1] & 0x01) == 1; // last bit and we transform it into a bool

        Header {
            id_dest,
            id_src,
            is_ack,
            id_message,
            seq_number,
        }
    }
    pub fn get_id_dest(&self) -> u8 {
        self.id_dest
    }
    pub fn get_id_src(&self) -> u8 {
        self.id_src
    }
    pub fn get_is_ack(&self) -> bool {
        self.is_ack
    }
    pub fn get_id_message(&self) -> u8 {
        self.id_message
    }
    pub fn get_seq_number(&self) -> u8 {
        self.seq_number
    }
}
