use core::borrow::BorrowMut;
use core::cell::{Ref, RefCell, RefMut};
use core::mem::MaybeUninit;
use embedded_hal::serial::{Read, Write};

use crate::motors::motor_group::MotorGroup;

mod motor_group;
mod communication;

// Size must be inferior to 32
const GROUP_SIZE : usize = 20;

// const INIT_MOTORGROUP : Option<RefCell<MotorGroup>> = None;

struct Motors<'a, Tx: Write<u8>, Rx : Read<u8>> {
    tx: &'a mut Tx,
    rx: &'a mut Rx,
    groups: [Option<RefCell<MotorGroup<'a, Tx, Rx>>>; GROUP_SIZE]
}

impl <'a, Tx: Write<u8>, Rx : Read<u8>> Motors<'a, Tx, Rx> {


    pub fn new(tx : &'a mut Tx, rx : &'a mut Rx) -> Motors<'a, Tx, Rx> {

        Motors{
            tx: (tx),
            rx: (rx),
            groups : Default::default()
            // groups: [INIT_MOTORGROUP; GROUP_SIZE]
        }
    }


    pub fn new_group(&mut self) -> RefMut<'a, MotorGroup<Tx, Rx>> {

        let mut i = 0;


        // Iteration throught groups and stop on the first empty cell to create a new group
        while self.groups[i].borrow_mut().is_none() {
            i+=1;
        }

        let mut group = MotorGroup::new(self.tx,self.rx);

        // RefCell is to share memory
        self.groups[i] = Some(RefCell::new(group)); // Add the new group to the array
        self.groups[i].as_ref().unwrap().borrow_mut()   // Returns a reference of the group

    }

    pub fn number_of_groups(&mut self) -> u8 {
        let mut i = 0;
        while self.groups[i].borrow_mut().is_none() {
            i+=1;
        }

        i as u8
    }
}
