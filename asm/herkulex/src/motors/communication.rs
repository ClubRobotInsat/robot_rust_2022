use core::borrow::{Borrow, BorrowMut};
use core::ops::Deref;
use drs_0x01::builder::HerkulexMessage;
use embedded_hal::serial::{Read, Write};
use nb::block;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::serial::{Rx, Tx};
use stm32f1xx_hal::{
    pac::interrupt,
    pac::USART1
};

const BUFF_SIZE : usize = 20;
static mut BUFF_INDEX : usize = 0;
static  mut BUFFER: &mut [u8; BUFF_SIZE] = &mut [0; BUFF_SIZE];

static mut RX: Option<Rx<USART1>> = None;

struct Communication<'a,> {
    tx: &'a mut Tx<USART1, u8>,
}

impl <'a> Communication<'a>  {

    pub fn new(tx : &'a mut Tx<USART1, u8>, mut rx : Rx<USART1, u8>) -> Communication<'a> {

        let mut c = Communication{
            tx: tx,
        };

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::USART1);
        }

        rx.listen();
        rx.listen_idle();
        cortex_m::interrupt::free(|_| unsafe {
            RX.replace(rx);
        });
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
                received_message[i] = BUFFER[i];
            }
        }
        received_message
    }



}
#[interrupt]
fn USART1() {
    // INT_COUNTER += 1; // Count the number of interruptions

    // When a packet is received, there is at least 3 bytes : header, header, packet size
    let mut packet_size = 3;

    unsafe  {
        cortex_m::interrupt::free(|_| {
            if let Some(rx) = RX.as_mut() {

                // If it received a packet and we have not read it entirely yet
                while rx.is_rx_not_empty() || BUFF_INDEX < packet_size {
                    // Read the byte
                    if let Ok(w)= (rx.read()){

                        BUFFER[BUFF_INDEX] = w; // Fill the buffer

                        // If we read the packet size in the received packet
                        // it updates our packet_size to read all bytes from the received packet
                        if BUFF_INDEX == 2 {
                            packet_size = w  as usize;
                        }

                        // If the buffer is full, it rewrites at the beginning
                        if BUFF_INDEX >= BUFF_SIZE - 1 {
                            BUFF_INDEX = 0;
                        }

                        BUFF_INDEX += 1;
                    }

                    rx.listen_idle();
                }

                // Reset the buffer index position for the next packet
                if rx.is_idle() {
                    rx.unlisten_idle();
                    BUFF_INDEX = 0;
                }
            }
        })

    }
}