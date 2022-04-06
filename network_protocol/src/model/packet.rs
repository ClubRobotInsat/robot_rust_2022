use crate::{BYTES_LEFT_IN_PACKAGE, SendError, Write};
use crate::model::header::Header;

/// This is only for definition and fields should never be accessed nor modified directly
/// use the methods provided with the struct instead
pub struct Packet {
    header: Header,
    payload: [u8; BYTES_LEFT_IN_PACKAGE as usize],
}

impl Packet {
    pub fn new(header: Header,data: [u8; BYTES_LEFT_IN_PACKAGE as usize]) -> Packet {
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
