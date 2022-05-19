use crate::motors::communication::HerkulexCommunication;
use core::borrow::Borrow;
use core::cell::RefCell;
use cortex_m_semihosting::hprintln;

use crate::motors::motor_group::MotorGroup;

pub mod communication;
pub mod motor_group;

// Size must be inferior to 32
const GROUP_SIZE: usize = 20;

// const INIT_MOTORGROUP : Option<RefCell<MotorGroup>> = None;

pub struct Motors<Comm: HerkulexCommunication> {
    communication: RefCell<Comm>,
}

impl<'a, Comm: HerkulexCommunication> Motors<Comm> {
    pub fn new(comm: Comm) -> Motors<Comm> {
        Motors {
            communication: RefCell::new(comm),
        }
    }

    pub fn new_group(&self) -> MotorGroup<Comm> {
        hprintln!("Motors::new_group");
        MotorGroup::new(&self.communication)
    }
}
