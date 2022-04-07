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
pub enum SendError {
    DidntReceiveACK,
    SendFailed
}

#[derive(Debug, PartialEq)]
pub enum MessageCreationError {
    MessageTooLong,
    ParametersTooLong,
    SrcAndDestCanNotBeEqual,
    ACKCanNotContainData,
    SendFailed(SendError)
}

pub struct MessageSender<Tx: Write, Rx: Read> {
    host_id: u8,
    tx: Tx,
    rx: Rx,
    id_mess_counter: u8, // really a u3
    message_buffer: [Option<Message>; MAX_ID_MESS_LEN as usize],
}

impl <Tx: Write,Rx: Read> MessageSender<Tx,Rx> {
    pub fn new(host_id: u8, tx: Tx, rx: Rx) -> Result<MessageSender<Tx, Rx>,MessageCreationError> {
        if host_id > MAX_ID_LEN {
            return Err(MessageCreationError::ParametersTooLong);
        }
        const INIT : Option<Message> = None; // to tell the compiler that it is a constant value known at compile time
        Ok(MessageSender { host_id, tx, rx, id_mess_counter: 0, message_buffer: [INIT;7] })
    }

    pub fn send_message(&mut self,id_dest: u8, data: Vec<u8>) -> Result<(),MessageCreationError> {
        if id_dest > MAX_ID_LEN {
            return Err(MessageCreationError::ParametersTooLong);
        }
        if data.len() > BYTES_LEFT_IN_PACKAGE as usize {
            return Err(MessageCreationError::MessageTooLong);
        }
        let mut message = Message::new(self.id_mess_counter, id_dest, self.host_id, data)?;
        // if the send fails we return an error
        if let Err(e) = message.send(&mut self.tx) {
            return Err(MessageCreationError::SendFailed(e));
        }
        // if the message was send correctly we put it int he buffer
        self.message_buffer[self.id_mess_counter as usize] = Some(message);

        // if the send succeeds we update the message id
        self.id_mess_counter = (self.id_mess_counter + 1)%(MAX_ID_MESS_LEN +1); // to make it fit in a u3
        // todo if message sent successfully remove it from the buffer
        Ok(())
    }

    pub fn get_host_id(&self) -> u8 {
        self.host_id & 0x0F // return only the firsts for bits as it's the max it can fit
    }
}