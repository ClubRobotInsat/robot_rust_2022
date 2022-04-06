extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;
use crate::{BYTES_LEFT_IN_PACKAGE, MAX_ID_LEN, MessageCreationError, SendError, Write};
use crate::model::header::Header;
use crate::model::packet::Packet;
use crate::model::protocol_constants::*;

/// maximum numbers of packets to send of a single message limited by the size of the sequence number
#[allow(dead_code)]
pub struct Message {
    id: u8,
    id_dest: u8,
    id_src: u8,
    data: Vec<u8>,
    ack_received: Vec<bool>
}

impl Message {
    pub fn new(id: u8, id_dest: u8, id_src: u8, data: Vec<u8>) -> Result<Message,MessageCreationError> {
        if id_dest == id_src {
            return Err(MessageCreationError::SrcAndDestCanNotBeEqual);
        }
        if id_dest > MAX_ID_LEN || id_src > MAX_ID_LEN {
            return Err(MessageCreationError::ParametersTooLong);
        }
        if data.len() > (BYTES_LEFT_IN_PACKAGE * MAX_SEQ_NUMBER_LEN) as usize {
            return Err(MessageCreationError::MessageTooLong)
        }

        let data_len = data.len();
        Ok(Message { id, id_dest, id_src, data, ack_received: vec![false; data_len] })
    }

    fn fill_data_with_zeros(&mut self, nb_of_0_to_add: usize) {
        for _ in 0..=nb_of_0_to_add {
            self.data.push(0); // we add 0's to fill the remaining message
        }
    }

    pub fn send<Tx: Write>(&mut self, tx: &mut Tx) -> Result<(),SendError> {
        let remainder = self.data.len()%BYTES_LEFT_IN_PACKAGE as usize;
        if  remainder == 0 { // add 0 to the end of the message
            self.fill_data_with_zeros(remainder)
        }
        // we can create an exact number of messages
        let seq_numbers = (0..self.data.len()/BYTES_LEFT_IN_PACKAGE as usize).rev();
        for i in seq_numbers {
            let header = Header::new(
                self.id_dest,
                self.id_src,
                false,
                self.id,
                u8::try_from(i).unwrap() // can't fail as the message is at max 90 bytes long which fits in a u8
            ).unwrap(); // we already checked the validity of the data in the new function of message so it can't crash
            let mut data:[u8; 6] = [0; 6];
            data[..6].clone_from_slice(&self.data[i..(6 + i)]);
            Packet::new(header,data).send(tx)?;

        }
        Err(SendError::SendFailed) // err

    }
}
