use crate::protocol::packet::Packet;
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
