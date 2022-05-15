use core::borrow::BorrowMut;
use core::ops::Deref;
use drs_0x01::builder::HerkulexMessage;
use embedded_hal::serial::{Read, Write};
use nb::block;
use stm32f1xx_hal::serial::{Rx, Tx};
use stm32f1xx_hal::stm32::USART1;

const BUFF_SIZE : usize = 20;

struct Communication<'a,> {
    tx: &'a mut Tx<USART1, u8>,
    rx: &'a mut Rx<USART1, u8>,
    RX : Option<Rx<USART1, u8>>,
    rx_buffer: [u8; BUFF_SIZE],
}

impl <'a> Communication<'a>  {

    pub fn new(tx : &'a mut Tx<USART1, u8>, rx : &'a mut Rx<USART1, u8>) -> Communication<'a> {

        let c = Communication{
            tx: tx,
            rx: rx,
            RX: None,
            rx_buffer: [0; BUFF_SIZE]
        };

        c
    }

    fn send_message(self, msg : HerkulexMessage) {
        for b in &msg {
            block!(self.tx.write(*b)); // Why no unwrap ?
            // block!(self.tx.write(*b)).unwrap_infallible();
        }
    }

    fn read_message(self) -> [u8; BUFF_SIZE] {
        cortex_m::asm::wfi();

        let mut received_message: [u8; BUFF_SIZE] = [0; BUFF_SIZE];
        for i in 0..BUFF_SIZE {
            unsafe {
                received_message[i] = self.rx_buffer[i];
            }
        }
        received_message

    }
}