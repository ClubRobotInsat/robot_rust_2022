use crate::protocol::errors::ProtocolError;
use crate::protocol::header::Header;
use crate::protocol::packet::Packet;
use crate::protocol::{CanId, MessageId, SeqId, MAX_MESSAGE_LEN, MAX_SEQ_NUMBER, PACKET_DATA_SIZE};
use cortex_m_semihosting::hprintln;
use heapless::Vec;
use stm32f1xx_hal::time::ms;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageId,
    id_dest: CanId,
    id_src: CanId,
    data: Vec<[u8; PACKET_DATA_SIZE], MAX_SEQ_NUMBER>,
    ack_received: Vec<bool, MAX_SEQ_NUMBER>,
}

impl Message {
    pub fn new(
        id: MessageId,
        id_dest: CanId,
        id_src: CanId,
        data: &[u8],
    ) -> Result<Message, ProtocolError> {
        if id_dest == id_src {
            Err(ProtocolError::SrcAndDestCanNotBeEqual)
        } else if data.len() > MAX_MESSAGE_LEN {
            Err(ProtocolError::MessageTooLong)
        } else {
            let packet_count = (data.len() / PACKET_DATA_SIZE as usize) + 1;

            let mut ack_received_vec = Vec::<bool, MAX_SEQ_NUMBER>::new();
            for _ in 0..packet_count {
                ack_received_vec.push(false).unwrap(); // cant crashas Vec always bigger than  sata_len
            }

            let mut original_data: Vec<[u8; PACKET_DATA_SIZE], MAX_SEQ_NUMBER> = Vec::new();
            for i in 0..(packet_count - 1) {
                // Vérifié au dessus, ne peut pas paniquer
                original_data.push([0u8; PACKET_DATA_SIZE]).unwrap();
                original_data[i]
                    .copy_from_slice(&data[i * PACKET_DATA_SIZE..(i + 1) * PACKET_DATA_SIZE]);
            }
            // Vérifié au dessus, ne peut pas paniquer
            original_data.push([0u8; PACKET_DATA_SIZE]).unwrap();
            for i in 0..PACKET_DATA_SIZE {
                original_data[packet_count - 1][i] =
                    if (packet_count - 1) * PACKET_DATA_SIZE + i < data.len() {
                        data[(packet_count - 1) * PACKET_DATA_SIZE + i]
                    } else {
                        0u8
                    }
            }

            Ok(Message {
                id,
                id_dest,
                id_src,
                data: original_data,
                ack_received: ack_received_vec,
            })
        }
    }

    pub fn mark_ack_as_received(&mut self, seq_num: SeqId) {
        let mess_len: usize = self.ack_received.len();
        match self.ack_received.get(mess_len - usize::from(seq_num) - 1) {
            None => {}
            Some(_) => self.ack_received[mess_len - usize::from(seq_num) - 1] = true,
        }
    }

    pub fn get_next_packet_to_send(&mut self) -> Result<Option<Packet>, ProtocolError> {
        match self.ack_received.iter().position(|b| !(*b)) {
            Some(index_packet) => {
                let header = Header::new(
                    self.id_dest,
                    self.id_src,
                    false,
                    self.id,
                    SeqId::new(self.ack_received.len() - index_packet - 1)?,
                )?;

                let mut payload = [0u8; PACKET_DATA_SIZE];
                payload.copy_from_slice(&self.data[index_packet]);

                Ok(Some(Packet { header, payload }))
            }
            None => Ok(None),
        }
    }

    /// Return true if all ACK of the different packets have been received
    pub fn all_ack_received(&self) -> bool {
        self.ack_received.iter().all(|b| *b)
    }
}
