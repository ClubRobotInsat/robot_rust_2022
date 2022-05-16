use core::borrow::BorrowMut;
use core::cell::{RefCell, RefMut};
use core::ops::DerefMut;
use drs_0x01::builder::HerkulexMessage;
use embedded_hal::serial::{Read, Write};
use stm32f1xx_hal::serial::{Rx, Tx};
use stm32f1xx_hal::{pac::USART1};
use crate::motors::communication::HerkulexCommunication;

pub struct Motor<'a, Comm : HerkulexCommunication> {
    communication : &'a RefCell<Comm>,
    id : u8
}

impl <'a, Comm : HerkulexCommunication> Motor<'_, Comm> {
    /// New Motor
    /// # Arguments
    /// id du moteur
    pub fn new( id : u8, communication : &'a RefCell<Comm>) -> Motor<'a, Comm>{
        Motor {
            id,
            communication :  communication
        }
    }

    pub fn getTemp(self) {
        self.communication.borrow_mut().read_message();
    }
}