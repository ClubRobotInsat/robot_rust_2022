use core::borrow::BorrowMut;
use core::ops::Deref;
use drs_0x01::builder::HerkulexMessage;
use embedded_hal::serial::{Read, Write};
use nb::block;

const BUFF_SIZE : usize = 20;

struct Communication<'a, Tx: Write<u8>, Rx : Read<u8>> {
    tx: &'a mut Tx,
    rx: &'a mut Rx,
    RX : Option<Rx>,
    rx_buffer: [u8; BUFF_SIZE],
}

impl <'a, Tx: Write<u8>, Rx : Read<u8>> Communication<'a, Tx, Rx>  {

    pub fn new(tx : &'a mut Tx, rx : &'a mut Rx) -> Communication<'a, Tx, Rx> {

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