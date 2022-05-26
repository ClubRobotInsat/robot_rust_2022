use crate::protocol::errors::ProtocolError;
use crate::protocol::header::Header;
use crate::protocol::{CAN_PACKET_SIZE, PACKET_DATA_SIZE};

/// This is only for definition and fields should never be accessed nor modified directly
/// use the methods provided with the struct instead
#[derive(Debug, Clone)]
pub struct Packet {
    pub header: Header,
    pub payload: [u8; PACKET_DATA_SIZE],
}

impl Packet {
    pub fn new(header: Header, data: [u8; PACKET_DATA_SIZE]) -> Packet {
        Packet {
            header,
            payload: data,
        }
    }
}

impl TryInto<[u8; CAN_PACKET_SIZE]> for &Packet {
    type Error = ProtocolError;

    fn try_into(self) -> Result<[u8; CAN_PACKET_SIZE], Self::Error> {
        let mut packet = [0u8; CAN_PACKET_SIZE];
        let slice: [u8; 2] = (&self.header).try_into()?;
        packet[0..2].copy_from_slice(&slice);
        packet[2..].copy_from_slice(&self.payload);
        Ok(packet)
    }
}

impl TryInto<[u8; CAN_PACKET_SIZE]> for Packet {
    type Error = ProtocolError;

    fn try_into(self) -> Result<[u8; CAN_PACKET_SIZE], Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<[u8; CAN_PACKET_SIZE]> for &mut Packet {
    type Error = ProtocolError;

    fn try_into(self) -> Result<[u8; CAN_PACKET_SIZE], Self::Error> {
        (&*self).try_into()
    }
}

impl TryFrom<&[u8; CAN_PACKET_SIZE]> for Packet {
    type Error = ProtocolError;

    fn try_from(value: &[u8; CAN_PACKET_SIZE]) -> Result<Self, Self::Error> {
        let mut payload = [0u8; 6];
        payload.copy_from_slice(&value[2..]);
        Ok(Packet {
            header: Header::try_from(value)?,
            payload,
        })
    }
}
