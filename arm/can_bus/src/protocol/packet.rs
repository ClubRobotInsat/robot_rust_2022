use crate::protocol::header::Header;
use crate::protocol::{BYTES_LEFT_IN_PACKAGE, CAN_PACKET_SIZE};
use cortex_m_semihosting::hprintln;

/// This is only for definition and fields should never be accessed nor modified directly
/// use the methods provided with the struct instead
///
#[derive(Debug, Clone)]
pub struct Packet {
    pub header: Header,
    pub payload: [u8; BYTES_LEFT_IN_PACKAGE],
}

impl Packet {
    pub fn new(header: Header, data: [u8; BYTES_LEFT_IN_PACKAGE]) -> Packet {
        Packet {
            header,
            payload: data,
        }
    }

    pub fn new_from_binary_array(data: &[u8; CAN_PACKET_SIZE]) -> Packet {
        let header = Header::new_from_binary_array(data[0..2].try_into().unwrap());
        let mut payload: [u8; 6] = [0; 6];
        payload.copy_from_slice(&data[2..]);
        Packet { header, payload }
    }

    pub fn get_packet_as_binary_array(&self) -> [u8; CAN_PACKET_SIZE as usize] {
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

    pub fn get_seq_num(&self) -> u8 {
        self.header.seq_number
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
