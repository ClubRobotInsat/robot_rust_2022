use crate::protocol::errors::ProtocolError;
use crate::protocol::header::Header;
use crate::protocol::packet::Packet;
use crate::protocol::{
    CanId, MessageId, SeqId, BYTES_LEFT_IN_PACKAGE, MAX_MESS_LEN, MAX_SEQ_NUMBER,
};
use cortex_m_semihosting::hprintln;
use heapless::Vec;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageId,
    pub id_dest: CanId,
    pub id_src: CanId,
    pub data: Vec<u8, MAX_MESS_LEN>,
    actual_sec_num: u8,
    pub ack_received: Vec<bool, MAX_SEQ_NUMBER>,
}

impl Message {
    pub fn new(
        id: MessageId,
        id_dest: CanId,
        id_src: CanId,
        data: Vec<u8, MAX_MESS_LEN>,
    ) -> Result<Message, ProtocolError> {
        if id_dest == id_src {
            return Err(ProtocolError::SrcAndDestCanNotBeEqual);
        }
        if data.len() / BYTES_LEFT_IN_PACKAGE as usize
            > (BYTES_LEFT_IN_PACKAGE * MAX_SEQ_NUMBER) as usize
        {
            return Err(ProtocolError::MessageTooLong);
        }

        let data_len = (data.len() / BYTES_LEFT_IN_PACKAGE as usize) + 1;
        //vec![false; data_len]
        let mut vec = Vec::<bool, MAX_SEQ_NUMBER>::new();
        for _ in 0..data_len {
            vec.push(false).unwrap(); // cant crashas Vec always bigger than  sata_len
        }
        // vec[0] = true;

        Ok(Message {
            id,
            id_dest,
            id_src,
            data,
            actual_sec_num: 0,
            ack_received: vec,
        })
    }

    pub(crate) fn mark_ack_as_received(&mut self, seq_num: SeqId) {
        // TODO: c'est bizzare
        let mess_len: usize = self.ack_received.len();
        match self.ack_received.get(mess_len - usize::from(seq_num)) {
            None => {}
            Some(_) => self.ack_received[mess_len - usize::from(seq_num)] = true,
        }
    }

    fn fill_data_with_zeros(&mut self, nb_of_0_to_add: usize) {
        for _ in 0..=nb_of_0_to_add {
            self.data.push(0).unwrap(); // we add 0's to fill the remaining message
        }
    }

    pub fn get_next_packet_to_send(&mut self) -> Result<Option<Vec<u8, 8>>, ProtocolError> {
        let remainder = self.data.len() % BYTES_LEFT_IN_PACKAGE as usize;
        if remainder != 0 {
            // add 0 to the end of the message
            self.fill_data_with_zeros(remainder + 1);
        }
        hprintln!("filled 0's").ok();
        let max_seq_num = self.data.len() / BYTES_LEFT_IN_PACKAGE as usize;
        // we can create an exact number of messages
        let seq_numbers = 0..(max_seq_num as usize); //(0..self.data.len()/BYTES_LEFT_IN_PACKAGE as usize).rev();
        let m: u32 = max_seq_num.try_into().unwrap();
        if self.all_ack_received() {
            return Ok(None);
        }

        hprintln!(
            "[{}]pre if: acks {:?}, dlen: {:?}",
            self.actual_sec_num,
            self.ack_received.clone(),
            max_seq_num
        )
        .ok();
        let len: u8 = self.ack_received.len().try_into().unwrap();
        if !self.ack_received[self.actual_sec_num as usize] {
            hprintln!("entered if").ok();
            let i = self.actual_sec_num as usize;
            // if the packet has already been received by the other STM dont' resend it

            let header = Header::new(
                self.id_dest,
                self.id_src,
                false,
                self.id, // TODO: Pourquoi c'est moche
                SeqId::new(max_seq_num - 1 - i)?,
                // can't fail as the message is at max 90 bytes long which fits in a u8
            )
            .unwrap(); // we already checked the validity of the data in the new function of message so it can't crash
            hprintln!("header created").ok();
            let mut data: [u8; BYTES_LEFT_IN_PACKAGE as usize] =
                [0; BYTES_LEFT_IN_PACKAGE as usize];
            // on decale chaque fois de la taille du message (6 bytes)
            hprintln!("array data: {:?}", self.data).ok();

            for n in 0..6 {
                data[n] = match self.data.get((self.actual_sec_num * 6) as usize + n) {
                    None => {
                        hprintln!("index err {:?}", n).ok();
                        0
                    }
                    Some(x) => *x,
                };
            }
            // data[..BYTES_LEFT_IN_PACKAGE as usize].clone_from_slice(
            //     &self.data[(i * BYTES_LEFT_IN_PACKAGE as usize)
            //         ..((BYTES_LEFT_IN_PACKAGE as usize + i * BYTES_LEFT_IN_PACKAGE as usize)
            //             as usize)],
            // );
            hprintln!("cloned from slice");

            // let x: Box<POOL, [u8; 8]> = POOL::alloc().unwrap();
            // hprintln!("actual seq num:{}", self.actual_sec_num);
            self.actual_sec_num = (self.actual_sec_num + 1) % len;
            return Ok(Some(
                Vec::from_slice(&Packet::new(header, data).get_packet_as_binary_array()).unwrap(),
            ));
        }
        self.actual_sec_num = (self.actual_sec_num + 1) % len;
        Ok(None)
        // this means it was sent successfully but we don't know if it was received we need to check the acks
        // Ok(())
    }

    pub fn process_msg(&mut self, msg: Packet) {
        self.ack_received[usize::from(msg.header.get_seq_number())] = true;
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
