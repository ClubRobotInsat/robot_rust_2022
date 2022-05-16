use core::borrow::{Borrow, BorrowMut};
use core::cell::{RefCell, RefMut};
use core::ops::DerefMut;
use embedded_hal::serial::{Read, Write};
use stm32f1xx_hal::serial::{Rx, Tx};
use stm32f1xx_hal::{pac::USART1};
use crate::motors::communication::HerkulexCommunication;
use crate::motors::motor_group::motor::Motor;
pub mod motor;

const GROUP_SIZE : usize = 20;

// const INIT_MOTOR : Option<RefCell<Motor>> = None;

pub struct MotorGroup<'a, Comm : HerkulexCommunication> {
    communication : &'a RefCell<Comm>,
    motors : [RefCell<Option<Motor<'a, Comm>>>; GROUP_SIZE]
}

impl <'a, Comm : HerkulexCommunication> MotorGroup<'a, Comm> {
    pub fn new(comm : &'a RefCell<Comm>) -> MotorGroup<'a, Comm>{
        MotorGroup {
            communication: (comm),
            motors : Default::default()
            // motors: [INIT_MOTOR; GROUP_SIZE]
        }
    }

    pub fn new_motor(&self, id : u8) -> &RefCell<Option<Motor<'a, Comm>>> {
        let mut i = 0;

        // Iteration throught groups and stop on the first empty cell to create a new motor
        while self.motors[i].borrow().is_none() {
            i+=1;
        }

        let mut motor = Motor::new(id, & self.communication);

        // RefCell is to share memory
        self.motors[i].borrow_mut().get_or_insert(motor); // Add the new group to the array
        //self.motors[i].as_ref().unwrap().borrow_mut()   // Returns a reference of the group
        self.motors.get(i).unwrap()

    }
}