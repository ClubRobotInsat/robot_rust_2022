use heapless::Vec;
use crate::{BUFFER_SIZE, BYTES_LEFT_IN_PACKAGE, MAX_ID_LEN, MessageCreationError, SendError, Write};
use crate::model::header::Header;
use crate::model::packet::Packet;
use crate::model::protocol_constants::*;

/// maximum numbers of packets to send of a single message limited by the size of the sequence number
#[allow(dead_code)]
#[derive(Debug)]
pub struct Message {
    id: u8,
    id_dest: u8,
    id_src: u8,
    pub data: Vec<u8, BUFFER_SIZE>,
    pub ack_received: Vec<bool, BUFFER_SIZE>
}

impl Message {
    pub fn new(id: u8, id_dest: u8, id_src: u8, data: Vec<u8, BUFFER_SIZE>) -> Result<Message,MessageCreationError> {
        if id_dest == id_src {
            return Err(MessageCreationError::SrcAndDestCanNotBeEqual);
        }
        if id_dest > MAX_ID_LEN || id_src > MAX_ID_LEN {
            return Err(MessageCreationError::ParametersTooLong);
        }
        if data.len()/BYTES_LEFT_IN_PACKAGE as usize > (BYTES_LEFT_IN_PACKAGE * MAX_SEQ_NUMBER_LEN) as usize {
            return Err(MessageCreationError::MessageTooLong)
        }

        let data_len = data.len()/BYTES_LEFT_IN_PACKAGE as usize;
        //vec![false; data_len]
        let mut vec =Vec::<bool,BUFFER_SIZE>::new();
        for i in 0..data_len {
            vec.push(false).unwrap(); // cant crashas Vec always bigger than  sata_len
        }

        Ok(Message { id, id_dest, id_src, data, ack_received: vec})
    }

    fn fill_data_with_zeros(&mut self, nb_of_0_to_add: usize) {
        for _ in 0..=nb_of_0_to_add {
            self.data.push(0).unwrap(); // we add 0's to fill the remaining message
        }
    }

    pub fn send<Tx: Write>(&mut self, tx: &mut Tx) -> Result<(),SendError> {
        let remainder = self.data.len()%BYTES_LEFT_IN_PACKAGE as usize;
        if  remainder == 0 { // add 0 to the end of the message
            self.fill_data_with_zeros(remainder)
        }
        let max_seq_num = self.data.len()/BYTES_LEFT_IN_PACKAGE as usize;
        // we can create an exact number of messages
        let seq_numbers = 0..(max_seq_num as usize);  //(0..self.data.len()/BYTES_LEFT_IN_PACKAGE as usize).rev();
        for i in seq_numbers {
            // if the packet has already been received by the other STM dont' resend it
            if self.ack_received[i] { continue }

            let header = Header::new(
                self.id_dest,
                self.id_src,
                false,
                self.id,
                u8::try_from(max_seq_num -1).unwrap() - u8::try_from(i).unwrap() // can't fail as the message is at max 90 bytes long which fits in a u8
            ).unwrap(); // we already checked the validity of the data in the new function of message so it can't crash

            let mut data:[u8; BYTES_LEFT_IN_PACKAGE as usize] = [0; BYTES_LEFT_IN_PACKAGE as usize];
            // on decale chaque fois de la taille du message (6 bytes)
            data[..BYTES_LEFT_IN_PACKAGE as usize].clone_from_slice(&self.data[(i*BYTES_LEFT_IN_PACKAGE as usize)..((BYTES_LEFT_IN_PACKAGE as usize + i*BYTES_LEFT_IN_PACKAGE as usize) as usize)]);
            Packet::new(header,data).send(tx)?;
        }
        // this means it was sent successfully but we don't know if it was received we need to check the acks
        Ok(())
    }

    pub fn process_msg(&mut self, msg: Packet) {
        self.ack_received[msg.header.get_seq_number() as usize] = true;
    }

    /// Return true if all ACK of the different packets have been received
    pub fn all_ack_received(&self) -> bool {
       self.ack_received
           .clone()
           .into_iter()
           .reduce(|accum, item| accum && item)
           .unwrap()
    }
}
