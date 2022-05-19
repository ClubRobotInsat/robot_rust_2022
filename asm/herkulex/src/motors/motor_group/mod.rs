use crate::motors::communication::HerkulexCommunication;
use crate::motors::motor_group::motor::Motor;
use core::cell::RefCell;
pub mod motor;

const GROUP_SIZE: usize = 20;

// const INIT_MOTOR : Option<RefCell<Motor>> = None;

pub struct MotorGroup<'a, Comm: HerkulexCommunication> {
    communication: &'a RefCell<Comm>,
    //motors: [RefCell<Option<Motor<'a, Comm>>>; GROUP_SIZE],
}

impl<'a, Comm: HerkulexCommunication> MotorGroup<'a, Comm> {
    pub fn new(comm: &'a RefCell<Comm>) -> MotorGroup<'a, Comm> {
        MotorGroup {
            communication: (comm),
            //motors: Default::default(), // motors: [INIT_MOTOR; GROUP_SIZE]
        }
    }

    pub fn new_motor(&self, id: u8) -> Motor<'a, Comm> {
        Motor::new(id, &self.communication)
    }
}
