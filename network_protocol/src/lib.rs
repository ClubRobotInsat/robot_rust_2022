#![no_std]

mod model;

extern crate alloc;
use alloc::vec::Vec;
use crate::model::message::Message;
use crate::model::protocol_constants::*;


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

pub struct MessageSender<Tx: Write, Rx: Read> {
    host_id: u8,
    tx: Tx,
    rx: Rx,
    id_mess_counter: u8 // really a u4
}

impl <Tx: Write,Rx: Read> MessageSender<Tx,Rx> {
    pub fn new(host_id: u8, tx: Tx, rx: Rx) -> Result<MessageSender<Tx, Rx>,MessageCreationError> {
        if host_id > MAX_ID_LEN {
            return Err(MessageCreationError::ParametersTooLong);
        }
        Ok(MessageSender { host_id, tx, rx, id_mess_counter: 0 })
    }

    pub fn send_message(&mut self,id_dest: u8, data: Vec<u8>) -> Result<(),MessageCreationError> {
        if id_dest > MAX_ID_LEN {
            return Err(MessageCreationError::ParametersTooLong);
        }
        if data.len() > BYTES_LEFT_IN_PACKAGE as usize {
            return Err(MessageCreationError::MessageTooLong);
        }
        let mut message = Message::new(self.id_mess_counter, id_dest, self.host_id, data)?;
        message.send(&mut self.tx);
        self.id_mess_counter = (self.id_mess_counter + 1)%16; // to make it fit in a u4
        // todo if message sent successfully remove it from the buffer
        Ok(())
    }
    pub fn get_host_id(&self) -> u8 {
        self.host_id & 0x0F // return only the firsts for bits as it's the max it can fit
    }
}