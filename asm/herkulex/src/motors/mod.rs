use core::borrow::BorrowMut;
use core::cell::{Ref, RefCell, RefMut};
use core::mem::MaybeUninit;
use cortex_m::interrupt::Mutex;
use embedded_hal::serial::{Read, Write};
use stm32f1xx_hal::serial::{Rx, Tx};
use stm32f1xx_hal::{pac::USART1};
use crate::motors::communication::HerkulexCommunication;

use crate::motors::motor_group::MotorGroup;

pub mod motor_group;
pub mod communication;

// Size must be inferior to 32
const GROUP_SIZE : usize = 20;

// const INIT_MOTORGROUP : Option<RefCell<MotorGroup>> = None;

pub struct Motors<'a, Comm : HerkulexCommunication> {
    communication : RefCell<Comm>,
    groups: [RefCell<Option<MotorGroup<'a, Comm>>>; GROUP_SIZE]
}

impl <'a, Comm : HerkulexCommunication> Motors<'a, Comm> {


    pub fn new(comm : Comm) -> Motors<'a, Comm> {
        Motors{
            communication: RefCell::new(comm),
            groups : Default::default()
            // groups: [INIT_MOTORGROUP; GROUP_SIZE]
        }
    }


    pub fn new_group(&'a self) -> usize {
        let i = self.number_of_groups();
        self.groups[i].borrow_mut().get_or_insert(MotorGroup::new(&self.communication));
        i

            //= Some(RefCell::new(group)); // Add the new group to the array
        //let q = &self.groups.borrow_mut()[i];
        // Returns a reference of the group
        //q.borrow_mut().unwrap()

    }

    pub fn get_group(&self, id: usize) -> &RefCell<Option<MotorGroup<'a, Comm>>> {
        self.groups.get(id).unwrap()
    }

    pub fn number_of_groups(&self) -> usize {
        let mut i = 0;
        while self.groups[i].borrow().is_none() {
            i+=1;
        }

        i
    }
}
