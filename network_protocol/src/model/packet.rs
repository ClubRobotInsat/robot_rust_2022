use crate::{BYTES_LEFT_IN_PACKAGE, CAN_PACKET_SIZE, SendError, Write};
use crate::model::header::Header;

/// This is only for definition and fields should never be accessed nor modified directly
/// use the methods provided with the struct instead
///
#[derive(Debug, Clone)]
pub struct Packet {
    pub header: Header,
    pub payload: [u8; BYTES_LEFT_IN_PACKAGE as usize],
}

impl Packet {
    pub fn new(header: Header,data: [u8; BYTES_LEFT_IN_PACKAGE as usize]) -> Packet {
        Packet { header, payload: data, }
    }

    pub fn new_from_binary_array(data: &[u8; CAN_PACKET_SIZE as usize]) -> Packet {
        let header = Header::new_from_binary_array(data[0..2].try_into().unwrap());
        let payload:[u8;6] = data[2..=CAN_PACKET_SIZE as usize].try_into().unwrap();
        Packet { header, payload, }
    }

    pub(super) fn get_packet_as_binary_array(&self) -> [u8; CAN_PACKET_SIZE as usize] {
        let mut packet = [0u8; CAN_PACKET_SIZE as usize];
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
