use crate::protocol::errors::ProtocolError;
use crate::protocol::message_in_progress::MessageInProgress;
use crate::protocol::{CAN_PACKET_SIZE, PACKET_DATA_SIZE};
use crate::{CanId, Message, Packet, SeqId};
use core::mem::swap;
use heapless::Vec;

pub struct Protocol {
    pub host_id: CanId,
    pub received: Vec<Vec<u8, 96>, 8>,
    pub acks_to_send: Vec<[u8; 8], 8>,
    pub send_buff: Vec<Message, 8>,
    pub messages_in_progess: Vec<MessageInProgress, 8>,
}

impl Protocol {
    pub fn new(host_id: CanId) -> Result<Self, ProtocolError> {
        Ok(Protocol {
            host_id,
            acks_to_send: Vec::new(),
            received: Vec::new(),
            send_buff: Vec::new(),
            messages_in_progess: Vec::new(),
        })
    }

    fn process_ack_packet(&mut self, packet: Packet) {
        match self
            .send_buff
            .iter()
            .position(|m| m.id == packet.header.id_message)
        {
            None => {}
            Some(i) => {
                self.send_buff[i].mark_ack_as_received(packet.header.seq_number);
                if self.send_buff[i].all_ack_received() {
                    self.send_buff.swap_remove(i);
                }
            }
        }
    }

    pub fn process_data_packet(&mut self, packet: Packet) {
        self.send_ack(&packet);
        // TODO P0: I've supposed we maintain order it shouldnt be that way
        let mut message_already_exists = false;
        for i in &mut self.messages_in_progess {
            match i.buff.get(0) {
                None => {}
                Some(v) => {
                    if v.header.id_message == packet.header.id_message {
                        i.buff.push(packet.clone()).unwrap();
                        message_already_exists = true;
                    }
                }
            }
        }
        if !message_already_exists {
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

    pub fn process_raw_packet(&mut self, mess: [u8; 8]) -> Result<(), ProtocolError> {
        let packet = Packet::try_from(&mess)?;

        if packet.header.id_dest != self.host_id {
            Ok(())
        } else {
            if packet.header.is_ack {
                self.process_ack_packet(packet);
            } else {
                self.process_data_packet(packet);
            }
            for mess in self.get_finished_messages()? {
                self.received.push(mess).unwrap();
            }
            Ok(())
        }
    }

    pub fn send_ack(&mut self, packet_to_respond: &Packet) -> Result<(), ProtocolError> {
        let mut ack_header = packet_to_respond.header.clone();
        swap(&mut ack_header.id_src, &mut ack_header.id_dest);
        ack_header.is_ack = true;
        self.acks_to_send
            .push(Packet::new(ack_header, [0u8; PACKET_DATA_SIZE]).try_into()?)
            .unwrap();
        Ok(())
    }

    pub fn add_message_to_send_buff(&mut self, mes: Message) {
        self.send_buff.push(mes).expect("send mess full");
    }
    pub fn get_next_packet_to_send(&mut self) -> Result<Option<[u8; 8]>, ProtocolError> {
        if !self.acks_to_send.is_empty() {
            return Ok(Some(self.acks_to_send.pop().unwrap()));
        }

        Ok(match self.send_buff.get_mut(0) {
            Some(message) => match message.get_next_packet_to_send()? {
                Some(packet) => Some(packet.try_into()?),
                None => None,
            },
            None => None,
        })
    }

    fn get_finished_messages(&mut self) -> Result<Vec<Vec<u8, 96>, 8>, ProtocolError> {
        let mut res: Vec<Vec<u8, 96>, 8> = Vec::new();
        for mess in &mut self.messages_in_progess {
            let mut max_seq_num = SeqId::new(0)?;
            for paquet in &mut mess.buff {
                max_seq_num = max_seq_num.max(paquet.header.seq_number);
            }
            if SeqId::new(mess.buff.len() - 1)? == max_seq_num {
                let mut new_mes: Vec<u8, 96> = Vec::new();
                for p in &mut mess.buff {
                    let arr: [u8; CAN_PACKET_SIZE] = p.try_into()?;
                    for i in 2..8 {
                        new_mes.push(arr[i]).unwrap();
                    }
                }
                res.push(new_mes).unwrap();
            }
        }
        Ok(res)
    }
}
