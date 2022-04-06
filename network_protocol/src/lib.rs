#![no_std]

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;

const MAX_ID_LEN: u8 = 16 -1;
const MAX_SEQ_NUMBER_LEN: u8 = 16 -1;
const MAX_ID_MESS_LEN: u8 = 8 -1;
const BYTES_LEFT_IN_PACKAGE: u8 = 6;

/// Defines a struct which can receive data ( RX )
pub trait Read<Word = u8> {
    type Error;
    fn read(&mut self) -> Result<Word, Self::Error>;
}

/// Defines a struct which can send data ( TX )
pub trait Write<Word = u8> {
    type Error;
    fn write(&mut self, word: Word) -> Result<(), Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
}


#[derive(Debug, PartialEq)]
pub enum MessageCreationError {
    MessageTooLong,
    ParametersTooLong,
    SrcAndDestCanNotBeEqual,
    ACKCanNotContainData,
}
pub enum SendError {
    DidntReceiveACK,
    SendFailed
}

/// This is only for definition and fields should never be accessed nor modified directly
/// use the methods provided with the struct instead
struct Header {
    id_dest: u8, // really a u4
    id_src: u8, // really a u4
    is_ack: bool, // really a u1
    id_message: u8, // really a u3
    seq_number: u8, // really a u4
}

/// This is only for definition and fields should never be accessed nor modified directly
/// use the methods provided with the struct instead
struct Packet {
    header: Header,
    payload: [u8; BYTES_LEFT_IN_PACKAGE as usize],
}

/// maximum numbers of packets to send of a single message limited by the size of the sequence number
#[allow(dead_code)]
struct Message {
    id: u8,
    id_dest: u8,
    id_src: u8,
    data: Vec<u8>,
    ack_received: Vec<bool>
}

struct MessageSender<Tx: Write, Rx: Read> {
    host_id: u8,
    tx: Tx,
    rx: Rx,
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
    pub fn new(id_dest: u8, id_src: u8, is_ack: bool, id_message: u8, seq_number: u8) -> Result<Header,MessageCreationError> {

        if id_dest > MAX_ID_LEN || id_src > MAX_ID_LEN || id_message > MAX_ID_MESS_LEN || seq_number > MAX_SEQ_NUMBER_LEN {
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

impl Packet {
    fn new(header: Header,data: [u8; BYTES_LEFT_IN_PACKAGE as usize]) -> Packet {
        Packet { header, payload: data, }
    }

    fn get_packet_as_binary_array(&self) -> [u8; 6] {
        let mut packet = [0u8; 6];
        let id_dest_in_significant_bits = self.header.get_id_dest() << 4;
        let id_send_in_least_significant_bits = self.header.get_id_src();
        packet[0] = id_dest_in_significant_bits | id_send_in_least_significant_bits;

        let id_mess_in_significant_bit = self.header.get_id_message() << 5;
        let seq_number = self.header.get_seq_number() << 1;
        let is_ack = self.header.get_is_ack() ;
        packet[1] = id_mess_in_significant_bit | seq_number | is_ack as u8;

        packet[2..].copy_from_slice(&self.payload);
        packet
    }

    pub fn send<Tx: Write>(&mut self, tx: &mut Tx) -> Result<(),SendError> {
        for byte in self.get_packet_as_binary_array() {
            match tx.write(byte) {
                Ok(_) => {},
                Err(_) => return Err(SendError::SendFailed)
            }
        }
        Ok(())
    }
}

impl Message {
    fn new(id: u8, id_dest: u8, id_src: u8, data: Vec<u8>) -> Result<Message,MessageCreationError> {
        if id_dest == id_src {
            return Err(MessageCreationError::SrcAndDestCanNotBeEqual);
        }
        if id_dest > MAX_ID_LEN || id_src > MAX_ID_LEN {
            return Err(MessageCreationError::MessageTooLong);
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

impl <Tx: Write,Rx: Read> MessageSender<Tx,Rx> {
    pub fn new(host_id: u8, tx: Tx, rx: Rx) -> Result<MessageSender<Tx, Rx>,MessageCreationError> {
        if host_id > MAX_ID_LEN {
            return Err(MessageCreationError::ParametersTooLong);
        }
        Ok(MessageSender { host_id, tx, rx, })
    }

    pub fn send_message(&mut self,id_dest: u8, data: Vec<u8>) -> Result<(),MessageCreationError> {
        if id_dest > MAX_ID_LEN {
            return Err(MessageCreationError::ParametersTooLong);
        }
        if data.len() > BYTES_LEFT_IN_PACKAGE as usize {
            return Err(MessageCreationError::MessageTooLong);
        }
        message.send();
        // send message
        // if message sent successfully remove it from the buffer
        Ok(())
    }
}