use core::borrow::BorrowMut;
use core::cell::{RefCell, RefMut};
use embedded_hal::serial::{Read, Write};
use stm32f1xx_hal::serial::{Rx, Tx};
use stm32f1xx_hal::{pac::USART1};
use crate::motors::motor_group::motor::Motor;
mod motor;

const GROUP_SIZE : usize = 20;

// const INIT_MOTOR : Option<RefCell<Motor>> = None;

pub struct MotorGroup<'a> {
    tx : &'a mut Tx<USART1, u8>,
    rx : &'a mut Rx<USART1, u8>,
    motors : [Option<RefCell<Motor<'a>>>; GROUP_SIZE]
}

impl <'a> MotorGroup<'a> {
    pub fn new(tx : &'a mut Tx<USART1, u8>, rx : &'a mut Rx<USART1, u8>) -> MotorGroup<'a>{
        MotorGroup {
            tx : tx,
            rx : rx,
            motors : Default::default()
            // motors: [INIT_MOTOR; GROUP_SIZE]
        }
    }

    pub fn new_motor(&mut self, id : u8) -> RefMut<'a, Motor> {
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