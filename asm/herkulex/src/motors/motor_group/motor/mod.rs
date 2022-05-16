use crate::motors::communication::HerkulexCommunication;
use core::cell::RefCell;

pub struct Motor<'a, Comm: HerkulexCommunication> {
    communication: &'a RefCell<Comm>,
    id: u8,
}

impl<'a, Comm: HerkulexCommunication> Motor<'_, Comm> {
    /// New Motor
    /// # Arguments
    /// id du moteur
    pub fn new(id: u8, communication: &'a RefCell<Comm>) -> Motor<'a, Comm> {
        Motor {
            id,
            communication: communication,
        }
    }

    pub fn getTemp(self) {
        self.communication.borrow_mut().read_message();
    }

    fn set_speed(self, speed: u16) {}
}
