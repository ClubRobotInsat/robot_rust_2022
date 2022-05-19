#![allow(unused_imports)]
#![allow(dead_code)]

// use core::alloc::vec;
// use core::alloc::vec::Vec;
use core::intrinsics::transmute;
use heapless::Vec;
use crate::{BUFFER_SIZE, Message, Packet, SendError};
use crate::model::header::Header;
use super::super::super::Write;

struct Tx {
    buff: Vec<u8, BUFFER_SIZE >
}

impl Write for Tx {
    type Error = SendError;

    fn write(&mut self, word: u8) -> Result<(), Self::Error> {
        self.buff.push(word);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn single_packet_message_creation() {
    let mut tx = Tx {buff: Vec::new()};
    let mut sender = Message::new(2, 3, 4, Vec::<u8,BUFFER_SIZE>::from_slice(&[1, 2, 3, 4, 5, 6]).unwrap()).unwrap();
    sender.send(&mut tx).unwrap();

    assert_eq!(tx.buff, Packet::new(
        Header::new(3, 4, false, 2, 0)
            .unwrap(),
        [1, 2, 3, 4, 5, 6])
        .get_packet_as_binary_array()
    );
    sender.send(&mut tx).unwrap();
}

/// check if the message is correctly cut in packets
#[test]
fn multiple_packet_message_creation() {
    let mut tx = Tx {buff: Vec::new()};
    let mut sender = Message::new(2, 3, 4, Vec::<u8,BUFFER_SIZE>::from_slice(&[1,2,3,4,5,6,7,8,9,10,11,12]).unwrap()).unwrap();
    sender.send(&mut tx).unwrap();

    println!("ldkasvjblda");
    assert_eq!(tx.buff, Packet::new(
        Header::new(3, 4, false, 2, 1)
            .unwrap(),
        [1,2,3,4,5,6])
        .get_packet_as_binary_array()
        .into_iter()
        .chain(
        Packet::new(
            Header::new(3, 4, false, 2, 0)
                .unwrap(),
            [7,8,9,10,11,12])
            .get_packet_as_binary_array()
            .into_iter()
        ).collect::<Vec<u8,BUFFER_SIZE>>()
    );
}

/// Check if the packets from which we havent received the ACKs are resend.
#[test]
fn send_multiple_times_before_ack() {
    let mut tx = Tx {buff: Vec::new()};
    let mut sender = Message::new(2, 3, 4, Vec::<u8,BUFFER_SIZE>::from_slice(&[1,2,3,4,5,6,7,8,9,10,11,12]).unwrap()).unwrap();
    sender.send(&mut tx).unwrap();
    sender.send(&mut tx).unwrap();
    let packet1 = Packet::new(
        Header::new(3, 4, false, 2, 1)
            .unwrap(),
        [1,2,3,4,5,6]);
    let packet2 = Packet::new(
        Header::new(3, 4, false, 2, 0)
            .unwrap(),
        [7, 8, 9, 10, 11, 12]);

    assert_eq!(tx.buff,packet1
        .get_packet_as_binary_array()
        .into_iter()
        .chain(
            packet2
                .get_packet_as_binary_array()
                .into_iter()
        )
        .chain(
            packet1
                .get_packet_as_binary_array()
                .into_iter()
        )
        .chain(
            packet2
                .get_packet_as_binary_array()
                .into_iter()
        )
        .collect::<Vec<u8,BUFFER_SIZE>>()
    );
}

#[test]
fn receive_all_acks(){
    let mut tx = Tx {buff: Vec::new()};
    let mut sender = Message::new(2, 3, 4, Vec::<u8,BUFFER_SIZE>::from_slice(&[6,5,4,3,2,1, 1, 2, 3, 4, 5, 6]).unwrap()).unwrap();
    sender.send(&mut tx).unwrap();

    sender.process_msg(Packet::new(Header::new(3,4,true,2,1).unwrap(), [0;6]));
    sender.process_msg(Packet::new(Header::new(3,4,true,2,0).unwrap(), [0;6]));
    assert!(sender.all_ack_received());
}

#[test]
fn receive_only_one_ack() {
    let mut tx = Tx {buff: Vec::new()};
    let mut sender = Message::new(2, 3, 4, Vec::<u8,BUFFER_SIZE>::from_slice(&[6,5,4,3,2,1, 1, 2, 3, 4, 5, 6]).unwrap()).unwrap();
    sender.process_msg(Packet::new(Header::new(3,4,true,2,1).unwrap(), [0;6]));
    sender.send(&mut tx).unwrap();

    assert_eq!(tx.buff,Packet::new(
        Header::new(3, 4, false, 2, 1)
            .unwrap(),
        [6,5,4,3,2,1])
        .get_packet_as_binary_array())
}