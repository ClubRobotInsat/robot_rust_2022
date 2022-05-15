use core::borrow::BorrowMut;
use core::cell::{RefCell, RefMut};
use embedded_hal::serial::{Read, Write};
use crate::motors::motor_group::motor::Motor;
mod motor;

const GROUP_SIZE : usize = 20;

// const INIT_MOTOR : Option<RefCell<Motor>> = None;

pub struct MotorGroup<'a, Tx: Write<u8>, Rx : Read<u8>> {
    tx : &'a mut Tx,
    rx : &'a mut Rx,
    motors : [Option<RefCell<Motor<'a, Tx,Rx>>>; GROUP_SIZE]
}

impl <'a, Tx: Write<u8>, Rx : Read<u8>> MotorGroup<'a, Tx, Rx> {
    pub fn new(tx : &'a mut Tx, rx : &'a mut Rx) -> MotorGroup<'a, Tx,Rx>{
        MotorGroup {
            tx : tx,
            rx : rx,
            motors : Default::default()
            // motors: [INIT_MOTOR; GROUP_SIZE]
        }
    }

    pub fn new_motor(&mut self, id : u8) -> RefMut<'a, Motor<Tx, Rx>> {
        let mut i = 0;

        // Iteration throught groups and stop on the first empty cell to create a new motor
        while self.motors[i].borrow_mut().is_none() {
            i+=1;
        }

        let mut motor = Motor::new(id, self.tx,  self.rx);

        // RefCell is to share memory
        self.motors[i] = Some(RefCell::new(motor)); // Add the new group to the array
        self.motors[i].as_ref().unwrap().borrow_mut()   // Returns a reference of the group

    }
}