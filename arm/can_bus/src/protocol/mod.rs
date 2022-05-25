//

mod errors;
mod header;
mod packet;

use core::cmp::max;
use cortex_m_semihosting::hprintln;
use heapless::Vec;

pub const U4_MAX: usize = 2usize.pow(4) - 1;
pub const U3_MAX: usize = 2usize.pow(3) - 1;

pub const MAX_ID: usize = U4_MAX;
pub const MAX_SEQ_NUMBER: usize = U4_MAX;
pub const MAX_ID_MESS: usize = U3_MAX;
pub const BYTES_LEFT_IN_PACKAGE: usize = 6;
pub const CAN_PACKET_SIZE: usize = 8;
pub const MAX_MESS_LEN: usize = MAX_SEQ_NUMBER * 6;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: u8,
    pub id_dest: u8,
    pub id_src: u8,
    pub data: Vec<u8, MAX_MESS_LEN>,
    actual_sec_num: u8,
    pub ack_received: Vec<bool, MAX_SEQ_NUMBER_LEN_USIZE>,
}

impl Message {
    pub fn new(
        id: u8,
        id_dest: u8,
        id_src: u8,
        data: Vec<u8, MAX_MESS_LEN>,
    ) -> Result<Message, MessageCreationError> {
        if id_dest == id_src {
            return Err(MessageCreationError::SrcAndDestCanNotBeEqual);
        }
        if id_dest > MAX_ID || id_src > MAX_ID {
            return Err(MessageCreationError::ParametersTooLong);
        }
        if data.len() / BYTES_LEFT_IN_PACKAGE as usize
            > (BYTES_LEFT_IN_PACKAGE * MAX_SEQ_NUMBER) as usize
        {
            return Err(MessageCreationError::MessageTooLong);
        }

        let data_len = (data.len() / BYTES_LEFT_IN_PACKAGE as usize) + 1;
        //vec![false; data_len]
        let mut vec = Vec::<bool, MAX_SEQ_NUMBER_LEN_USIZE>::new();
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

    fn mark_ack_as_received(&mut self, seq_num: u8) {
        let seq_num = seq_num as usize;
        let mess_len: usize = self.ack_received.len();
        match self.ack_received.get(mess_len - seq_num) {
            None => {}
            Some(_) => self.ack_received[mess_len - seq_num] = true,
        }
    }

    fn fill_data_with_zeros(&mut self, nb_of_0_to_add: usize) {
        for _ in 0..=nb_of_0_to_add {
            self.data.push(0).unwrap(); // we add 0's to fill the remaining message
        }
    }

    pub fn get_next_packet_to_send(&mut self) -> Option<Vec<u8, 8>> {
        let remainder = self.data.len() % BYTES_LEFT_IN_PACKAGE as usize;
        if remainder != 0 {
            // add 0 to the end of the message
            self.fill_data_with_zeros(remainder + 1);
        }
        hprintln!("filled 0's").ok();
        let max_seq_num = (self.data.len() / BYTES_LEFT_IN_PACKAGE as usize);
        // we can create an exact number of messages
        let seq_numbers = 0..(max_seq_num as usize); //(0..self.data.len()/BYTES_LEFT_IN_PACKAGE as usize).rev();
        let m: u32 = max_seq_num.try_into().unwrap();
        if self.all_ack_received() {
            return None;
        }
        // while self.ack_received[(self.actual_sec_num % MAX_SEQ_NUMBER_LEN) as usize] {
        //     self.actual_sec_num = (self.actual_sec_num +1)%MAX_SEQ_NUMBER_LEN;
        // }

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
                self.id,
                u8::try_from(max_seq_num - 1).unwrap() - u8::try_from(i).unwrap(), // can't fail as the message is at max 90 bytes long which fits in a u8
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
            return Some(
                Vec::from_slice(&Packet::new(header, data).get_packet_as_binary_array()).unwrap(),
            );
        }
        self.actual_sec_num = (self.actual_sec_num + 1) % len;
        None
        // this means it was sent successfully but we don't know if it was received we need to check the acks
        // Ok(())
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

#[derive(Debug, Clone)]
pub struct MessageInProgress {
    pub buff: Vec<Packet, 8>,
}

impl MessageInProgress {
    pub fn new() -> Self {
        MessageInProgress { buff: Vec::new() }
    }
}

pub struct Messages {
    pub host_id: u8,
    pub received: Vec<Vec<u8, 96>, 8>,
    pub acks_to_send: Vec<[u8; 8], 8>,
    pub send_buff: Vec<Message, 8>,
    pub messages_in_progess: Vec<MessageInProgress, 8>,
}

impl Messages {
    pub fn new(host_id: u8) -> Result<Self, MessageCreationError> {
        if host_id > MAX_ID {
            return Err(MessageCreationError::ParametersTooLong);
        }
        Ok(Messages {
            host_id,
            acks_to_send: Vec::new(),
            received: Vec::new(),
            send_buff: Vec::new(),
            messages_in_progess: Vec::new(),
        })
    }
    pub fn process_message(&mut self, mess: [u8; 8]) {
        hprintln!("processing mess").ok();
        let packet = Packet::new_from_binary_array(&mess);
        hprintln!("interpreted packet").ok();
        if packet.get_dest_id() != self.host_id {
            return;
        }
        // the packet is for us
        if packet.is_ack() {
            hprintln!("is ack").ok();
            for mess in &mut self.send_buff {
                if mess.id == packet.get_message_id() {
                    mess.mark_ack_as_received(packet.get_ack_num());
                    hprintln!("marked as received").ok();
                }
            }
            for i in 0..self.send_buff.len() {
                if self.send_buff[i].all_ack_received() {
                    self.send_buff.swap_remove(i);
                }
            }
        } else {
            self.respond_ack(packet.clone());
            // TODO P0: I've supposed we maintain order it shouldnt be that way
            let mut message_already_exists = false;
            let len = self.messages_in_progess.len();
            for i in &mut self.messages_in_progess {
                hprintln!("message existed").ok();
                match i.buff.get(0) {
                    None => {
                        hprintln!("bizzar??");
                    }
                    Some(v) => {
                        if v.get_message_id() == packet.get_message_id() {
                            i.buff.push(packet.clone()).unwrap();
                            message_already_exists = true;
                        }
                    }
                }
            }
            if !message_already_exists {
                hprintln!("new message created").ok();
                self.messages_in_progess
                    .push(MessageInProgress::new())
                    .unwrap();
                self.messages_in_progess
                    .last_mut()
                    .unwrap()
                    .buff
                    .push(packet.clone())
                    .unwrap();
            }
        }
        for mess in self.get_finished_messages() {
            self.received.push(mess).unwrap();
        }
        // todo
    }
    pub fn respond_ack(&mut self, packet_to_respond: Packet) {
        let mut paquet =
            Packet::new_from_binary_array(&packet_to_respond.get_packet_as_binary_array());
        let src_id = paquet.get_src_id();
        paquet.header.id_src = paquet.header.id_dest;
        paquet.header.id_dest = src_id;
        paquet.header.is_ack = true;
        hprintln!("respond ack").ok();
        self.acks_to_send
            .push(paquet.get_packet_as_binary_array())
            .unwrap();
        hprintln!("respond ack unwruap").ok();
    }

    pub fn add_message_to_send_buff(&mut self, mes: Message) {
        self.send_buff.push(mes).expect("send mess full");
    }
    pub fn get_next_packet_to_send(&mut self) -> Option<Vec<u8, 8>> {
        hprintln!("packet send").ok();
        if !self.acks_to_send.is_empty() {
            return Some(Vec::from_slice(&self.acks_to_send.pop().unwrap()).unwrap());
        }
        self.send_buff.get_mut(0)?.get_next_packet_to_send()
    }

    fn get_finished_messages(&mut self) -> Vec<Vec<u8, 96>, 8> {
        let mut res: Vec<Vec<u8, 96>, 8> = Vec::new();
        for mess in &mut self.messages_in_progess {
            let mut max_seq_num: u8 = 0;
            for paquet in &mut mess.buff {
                max_seq_num = max(max_seq_num, paquet.get_seq_num());
            }
            hprintln!("found max: {}", max_seq_num).ok();
            if mess.buff.len() - 1 as usize == max_seq_num as usize {
                let mut new_mes: Vec<u8, 96> = Vec::new();
                for p in &mut mess.buff {
                    let arr = p.get_packet_as_binary_array();
                    for i in 2..8 {
                        new_mes.push(arr[i]).unwrap();
                    }
                }
                res.push(new_mes).unwrap();
            }
        }
        hprintln!("completed messages restunr: {:?}", res.clone()).ok();
        res
    }
}
