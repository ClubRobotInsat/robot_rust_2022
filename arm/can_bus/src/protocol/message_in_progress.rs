use crate::protocol::errors::ProtocolError;
use crate::protocol::packet::Packet;
use crate::protocol::{CanId, SeqId};
use crate::Message;
use cortex_m_semihosting::hprintln;
use heapless::Vec;

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
    pub host_id: CanId,
    pub received: Vec<Vec<u8, 96>, 8>,
    pub acks_to_send: Vec<[u8; 8], 8>,
    pub send_buff: Vec<Message, 8>,
    pub messages_in_progess: Vec<MessageInProgress, 8>,
}

impl Messages {
    pub fn new(host_id: CanId) -> Result<Self, ProtocolError> {
        Ok(Messages {
            host_id,
            acks_to_send: Vec::new(),
            received: Vec::new(),
            send_buff: Vec::new(),
            messages_in_progess: Vec::new(),
        })
    }

    pub fn process_raw_packet(&mut self, mess: [u8; 8]) -> Result<(), ProtocolError> {
        hprintln!("processing mess").ok();
        let packet = Packet::new_from_binary_array(&mess)?;
        hprintln!("interpreted packet").ok();
        if packet.header.id_dest != self.host_id {
            return Ok(());
        }
        // the packet is for us
        if packet.header.is_ack {
            hprintln!("is ack").ok();
            for mess in &mut self.send_buff {
                if mess.id == packet.header.id_message {
                    mess.mark_ack_as_received(packet.header.seq_number);
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
                        if v.header.id_message == packet.header.id_message {
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
        for mess in self.get_finished_messages()? {
            self.received.push(mess).unwrap();
        }
        Ok(())
        // todo
    }
    pub fn respond_ack(&mut self, packet_to_respond: Packet) -> Result<(), ProtocolError> {
        let mut paquet =
            Packet::new_from_binary_array(&packet_to_respond.get_packet_as_binary_array())?;
        let src_id = paquet.header.id_src;
        paquet.header.id_src = paquet.header.id_dest;
        paquet.header.id_dest = src_id;
        paquet.header.is_ack = true;
        hprintln!("respond ack").ok();
        self.acks_to_send
            .push(paquet.get_packet_as_binary_array())
            .unwrap();
        hprintln!("respond ack unwruap").ok();
        Ok(())
    }

    pub fn add_message_to_send_buff(&mut self, mes: Message) {
        self.send_buff.push(mes).expect("send mess full");
    }
    pub fn get_next_packet_to_send(&mut self) -> Result<Option<Vec<u8, 8>>, ProtocolError> {
        hprintln!("packet send").ok();
        if !self.acks_to_send.is_empty() {
            return Ok(Some(
                Vec::from_slice(&self.acks_to_send.pop().unwrap()).unwrap(),
            ));
        }
        match self.send_buff.get_mut(0) {
            Some(v) => v.get_next_packet_to_send(),
            None => Ok(None),
        }
    }

    fn get_finished_messages(&mut self) -> Result<Vec<Vec<u8, 96>, 8>, ProtocolError> {
        let mut res: Vec<Vec<u8, 96>, 8> = Vec::new();
        for mess in &mut self.messages_in_progess {
            let mut max_seq_num = SeqId::new(0)?;
            for paquet in &mut mess.buff {
                max_seq_num = max_seq_num.max(paquet.header.seq_number);
            }
            hprintln!("found max: {:?}", max_seq_num).ok();
            if SeqId::new(mess.buff.len() - 1)? == max_seq_num {
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
        Ok(res)
    }
}
