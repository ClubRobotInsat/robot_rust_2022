use crate::motors::communication::HerkulexCommunication;
use crate::motors::motor::Motor;
use core::cell::RefCell;
use cortex_m_semihosting::hprintln;

pub mod communication;
pub mod motor;

pub struct Motors<Comm: HerkulexCommunication> {
    communication: RefCell<Comm>,
}

impl<'a, Comm: HerkulexCommunication> Motors<Comm> {
    pub fn new(comm: Comm) -> Motors<Comm> {
        Motors {
            communication: RefCell::new(comm),
        }
    }

    pub fn new_motor(&self, id: u8) -> Motor<Comm> {
        Motor::new(id, &self.communication)
    }
}
