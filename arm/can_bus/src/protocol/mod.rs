//

use cortex_m_semihosting::hprintln;
use heapless::Vec;

pub const MAX_ID_LEN: u8 = 16 - 1;
pub const MAX_SEQ_NUMBER_LEN: u8 = 16 - 1;
pub const MAX_SEQ_NUMBER_LEN_USIZE: usize = 16 - 1;
pub const MAX_ID_MESS_LEN: u8 = 8 - 1;
pub const BYTES_LEFT_IN_PACKAGE: u8 = 6;
pub const CAN_PACKET_SIZE: u8 = 8;
pub const MAX_MESS_LEN: usize = (MAX_SEQ_NUMBER_LEN * 6) as usize;

#[derive(Debug, PartialEq)]
pub enum SendError {
    DidntReceiveACK,
    SendFailed,
}

#[derive(Debug, PartialEq)]
pub enum MessageCreationError {
    MessageTooLong,
    ParametersTooLong,
    SrcAndDestCanNotBeEqual,
    ACKCanNotContainData,
    SendFailed(SendError),
}

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
        if id_dest > MAX_ID_LEN
            || id_src > MAX_ID_LEN
            || id_message > MAX_ID_MESS_LEN
            || seq_number > MAX_SEQ_NUMBER_LEN
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

/// This is only for definition and fields should never be accessed nor modified directly
/// use the methods provided with the struct instead
///
#[derive(Debug, Clone)]
pub struct Packet {
    pub header: Header,
    pub payload: [u8; BYTES_LEFT_IN_PACKAGE as usize],
}

impl Packet {
    pub fn new(header: Header, data: [u8; BYTES_LEFT_IN_PACKAGE as usize]) -> Packet {
        Packet {
            header,
            payload: data,
        }
    }

    pub fn new_from_binary_array(data: &[u8; CAN_PACKET_SIZE as usize]) -> Packet {
        let header = Header::new_from_binary_array(data[0..2].try_into().unwrap());
        hprintln!("interpreted header").ok();
        let mut payload: [u8;6] = [0;6];
        hprintln!("data: {:?}", data.clone()).ok();
        for i in 0..6 {
            // hprintln!("i: {}", i).ok();
            payload[i] = data[2+i];
        }
        hprintln!("payload created").ok();
        // let payload: [u8; 6] = match data[2..].try_into() {
        //     Err(e) => {hprintln!("err {:?}", e).ok(); [0;6]},
        //     Ok(v) => v
        // };
        Packet { header, payload }
    }

    pub(super) fn get_packet_as_binary_array(&self) -> [u8; CAN_PACKET_SIZE as usize] {
        let mut packet = [0u8; CAN_PACKET_SIZE as usize];
        let id_dest_in_significant_bits = self.header.get_id_dest() << 4;
        let id_send_in_least_significant_bits = self.header.get_id_src();
        packet[0] = id_dest_in_significant_bits | id_send_in_least_significant_bits;

        let id_mess_in_significant_bit = self.header.get_id_message() << 5;
        let seq_number = self.header.get_seq_number() << 1;
        let is_ack = self.header.get_is_ack();
        packet[1] = id_mess_in_significant_bit | seq_number | is_ack as u8;

        packet[2..].copy_from_slice(&self.payload);
        packet
    }

    fn get_message_id(&self) -> u8 {
        self.header.get_id_message()
    }

    pub fn is_ack(&self) -> bool {
        self.header.is_ack
    }

    pub fn get_ack_num(&self) -> u8 {
        self.header.seq_number
    }

    pub fn get_src_id(&self) -> u8 {
        self.header.get_id_src()
    }

    pub fn get_dest_id(&self) -> u8 {
        self.header.id_dest
    }

    // pub fn send<Tx: Write>(&mut self, tx: &mut Tx) -> Result<(), SendError> {
    //     for byte in self.get_packet_as_binary_array() {
    //         match tx.write(byte) {
    //             Ok(_) => {}
    //             Err(_) => return Err(SendError::SendFailed),
    //         }
    //     }
    //     Ok(())
    // }
}

#[allow(dead_code)]
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
        if id_dest > MAX_ID_LEN || id_src > MAX_ID_LEN {
            return Err(MessageCreationError::ParametersTooLong);
        }
        if data.len() / BYTES_LEFT_IN_PACKAGE as usize
            > (BYTES_LEFT_IN_PACKAGE * MAX_SEQ_NUMBER_LEN) as usize
        {
            return Err(MessageCreationError::MessageTooLong);
        }

        let data_len = (data.len() / BYTES_LEFT_IN_PACKAGE as usize)+1;
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
        match self.ack_received.get( mess_len- seq_num) {
            None => {  }
            Some(_) => { self.ack_received[mess_len - seq_num] = true }
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
            self.fill_data_with_zeros(remainder +1);
        }
        hprintln!("filled 0's").ok();
        let max_seq_num = (self.data.len() / BYTES_LEFT_IN_PACKAGE as usize);
        // we can create an exact number of messages
        let seq_numbers = 0..(max_seq_num as usize); //(0..self.data.len()/BYTES_LEFT_IN_PACKAGE as usize).rev();
        let m:u32 = max_seq_num.try_into().unwrap();
        if self.all_ack_received() {
            return None;
        }
        // while self.ack_received[(self.actual_sec_num % MAX_SEQ_NUMBER_LEN) as usize] {
        //     self.actual_sec_num = (self.actual_sec_num +1)%MAX_SEQ_NUMBER_LEN;
        // }

        hprintln!("[{}]pre if: acks {:?}, dlen: {:?}",self.actual_sec_num ,self.ack_received.clone(), max_seq_num);
        let len: u8 = self.ack_received.len().try_into().unwrap();
        if !self.ack_received[self.actual_sec_num as usize] {
            hprintln!("entered if");
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
            hprintln!("header created");
            let mut data: [u8; BYTES_LEFT_IN_PACKAGE as usize] =
                [0; BYTES_LEFT_IN_PACKAGE as usize];
            // on decale chaque fois de la taille du message (6 bytes)
            hprintln!("array data: {:?}", self.data);

            for n in 0..6 {
                data[n] = match self.data.get((self.actual_sec_num*6) as usize + n) {
                    None => {hprintln!("index err {:?}",n); 0}
                    Some(x) => { *x}
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
            self.actual_sec_num = (self.actual_sec_num +1)%len;
            return Some(
                Vec::from_slice(&Packet::new(header, data).get_packet_as_binary_array()).unwrap(),
            );
        }
        self.actual_sec_num = (self.actual_sec_num +1)%len;
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
        MessageInProgress {
            buff: Vec::new()
        }
    }
}

pub struct Messages {
    pub host_id: u8,
    pub received: Vec<Message, 8>,
    pub acks_to_send: Vec<[u8;8],8>,
    pub send_buff: Vec<Message, 8>,
    pub messages_in_progess: Vec<MessageInProgress,8>
}

impl Messages {
    pub fn new(host_id: u8) -> Result<Self, MessageCreationError> {
        if host_id > MAX_ID_LEN {
            return Err(MessageCreationError::ParametersTooLong);
        }
        Ok(Messages {
            host_id,
            acks_to_send: Vec::new(),
            received: Vec::new(),
            send_buff: Vec::new(),
            messages_in_progess: Vec::new()
        })
    }
    pub fn process_message(&mut self, mess: [u8;8]) {
        hprintln!("processing mess").ok();
        let packet = Packet::new_from_binary_array(&mess);
        hprintln!("interpreted packet").ok();
        if packet.get_dest_id() != self.host_id { return; }
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
                match i.buff.get(0)  {
                    None => {hprintln!("bizzar??");}
                    Some(v) => {
                        if v.get_message_id() == packet.get_message_id() {
                        i.buff.push(packet.clone()).unwrap();
                        message_already_exists = true;
                    }}
                }

            }
            if !message_already_exists {
                self.messages_in_progess.push(MessageInProgress::new()).unwrap();
                self.messages_in_progess.last_mut().unwrap().buff.push(packet.clone()).unwrap();
            }
        }
        // todo
    }
    pub fn respond_ack(&mut self, packet_to_respond: Packet) {
        let mut paquet = Packet::new_from_binary_array(&packet_to_respond.get_packet_as_binary_array());
        let src_id = paquet.get_src_id();
        paquet.header.id_src = paquet.header.id_dest;
        paquet.header.id_dest = src_id;
        paquet.header.is_ack = true;
        hprintln!("respond ack").ok();
        self.acks_to_send.push(paquet.get_packet_as_binary_array()).unwrap();
        hprintln!("respond ack unwruap").ok();
    }

    pub fn add_message_to_send_buff(&mut self, mes: Message) {
        self.send_buff.push(mes).expect("send mess full");
    }
    pub fn get_next_packet_to_send(&mut self) -> Option<Vec<u8, 8>> {
        hprintln!("packet send").ok();
        if self.acks_to_send.len() != 0 {
            return Some(Vec::from_slice(&self.acks_to_send.pop().unwrap()).unwrap())
        }
        self.send_buff.get_mut(0)?.get_next_packet_to_send()
    }
}
