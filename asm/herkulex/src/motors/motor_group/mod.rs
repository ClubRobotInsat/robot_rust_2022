use crate::motors::communication::HerkulexCommunication;
use crate::motors::motor_group::motor::Motor;
use core::borrow::{Borrow, BorrowMut};
use core::cell::RefCell;
use core::ops::Deref;

pub mod motor;
use heapless::Vec;
const GROUP_SIZE: usize = 20;

// const INIT_MOTOR : Option<RefCell<Motor>> = None;

pub struct MotorGroup<'a, Comm: HerkulexCommunication> {
    communication: &'a RefCell<Comm>,
    //motors: [RefCell<Option<Motor<'a, Comm>>>; GROUP_SIZE],
    // motors: [RefCell<Option<Motor<'a, Comm>>>; GROUP_SIZE],
    motors: RefCell<Vec<Motor<'a, Comm>, GROUP_SIZE>>,
}

impl<'a, Comm: HerkulexCommunication> MotorGroup<'a, Comm> {
    pub fn new(comm: &'a RefCell<Comm>) -> MotorGroup<'a, Comm> {
        MotorGroup {
            communication: (comm),
            motors: Default::default(), // motors: [INIT_MOTOR; GROUP_SIZE]
        }
    }

    pub fn new_motor(&self, id: u8) -> Option<&Motor<'a, Comm>> {
        self.motors
            .borrow_mut()
            .push(Motor::new(id, &self.communication));
        // self.motors.push(Motor::new(id, &self.communication));
        // Motor::new(id, &self.communication)
        self.motors.borrow_mut().last()
    }

    fn get_motor(&self, id: usize) -> &Motor<'a, Comm> {
        self.motors.borrow().get(id).unwrap()
    }
}
