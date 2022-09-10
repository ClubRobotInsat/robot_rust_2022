//! Simple CAN example.
//! Requires a transceiver connected to PA11, PA12 (CAN1) or PB5 PB6 (CAN2).

#![no_main]
#![no_std]

use core::cell::RefCell;
use panic_halt as _;

use crate::pac::NVIC;
use crate::protocol::message::Message;
use crate::protocol::packet::Packet;
use crate::protocol::protocol::Protocol;
use crate::protocol::{CanId, MessageId, SeqId};
use bxcan::filter::Mask32;
use bxcan::Interrupt::Fifo0MessagePending;
use bxcan::{Frame, StandardId};
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nb::block;
use stm32f1::stm32f103::{Interrupt, CAN1};
use stm32f1xx_hal::can::Can;
use stm32f1xx_hal::timer::Timer;
use stm32f1xx_hal::{
    pac::{self, interrupt},
    prelude::*,
};

mod protocol;

const SENDER_ID: usize = 3;
const RECEIVER_ID: usize = 0;

static CAN: Mutex<RefCell<Option<bxcan::Can<Can<CAN1>>>>> = Mutex::new(RefCell::new(None));
static SENDER: Mutex<RefCell<Option<Protocol>>> = Mutex::new(RefCell::new(None));

#[allow(non_snake_case)]
#[interrupt]
fn USB_LP_CAN_RX0() {
    cortex_m::interrupt::free(|cs| {
        let mut can_lock = CAN.borrow(cs).borrow_mut();
        let mut can = can_lock.take().expect("Failed to lock can");
        match block!(can.receive()) {
            Ok(v) => {
                let read: [u8; 8] = <[u8; 8]>::try_from(v.data().unwrap().as_ref()).unwrap();
                hprintln!("Recv raw packet {:?}", read);
                let mut sender_lock = SENDER.borrow(cs).borrow_mut();
                let mut sender = sender_lock.take().expect("Failed to lock sender");
                sender.process_raw_packet(read).unwrap();
                sender_lock.replace(sender);
            }
            Err(e) => {
                hprintln!("err: {:?}", e).ok();
            }
        }
        can_lock.replace(can);
    });
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();

    let rx_can = gpioa.pa11.into_floating_input(&mut gpioa.crh);
    let tx_can = gpioa.pa12.into_alternate_push_pull(&mut gpioa.crh);

    // To meet CAN clock accuracy requirements an external crystal or ceramic
    // resonator must be used. The blue pill has a 8MHz external crystal.
    // Other boards might have a crystal with another frequency or none at all.
    let clocks = rcc.cfgr.use_hse(8.MHz()).freeze(&mut flash.acr);

    let mut can = {
        let can: Can<CAN1> = Can::new(dp.CAN1, dp.USB);

        can.assign_pins((tx_can, rx_can), &mut afio.mapr);

        bxcan::Can::builder(can)
            .set_bit_timing(0x001c_0003)
            .leave_disabled()
    };

    // Configure filters so that can frames can be received
    can.modify_filters().enable_bank(0, Mask32::accept_all());

    block!(can.enable_non_blocking()).unwrap();
    can.enable_interrupt(Fifo0MessagePending);

    cortex_m::interrupt::free(|cs| {
        CAN.borrow(cs).replace(Some(can));
        SENDER.borrow(&cs).replace(Some(
            Protocol::new(CanId::new(SENDER_ID as usize).unwrap()).unwrap(),
        ));
    });

    unsafe { NVIC::unmask(Interrupt::USB_LP_CAN_RX0) }

    // New Delay with timer
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();

    hprintln!("Début de l'envoi").ok();

    let mut sender = Protocol::new(CanId::new(SENDER_ID).unwrap()).unwrap();

    sender.add_message_to_send_buff(
        Message::new(
            MessageId::new(0).unwrap(),
            CanId::new(RECEIVER_ID).unwrap(),
            sender.host_id,
            &[1, 2, 3, 4, 5, 6, 7, 8],
        )
        .unwrap(),
    );
    sender.add_message_to_send_buff(
        Message::new(
            MessageId::new(2).unwrap(),
            CanId::new(RECEIVER_ID).unwrap(),
            sender.host_id,
            &[9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
        )
        .unwrap(),
    );

    cortex_m::interrupt::free(|cs| {
        let mut sender_lock = SENDER.borrow(cs).borrow_mut();
        sender_lock.replace(sender);
    });

    loop {
        let next_packet = cortex_m::interrupt::free(|cs| {
            let mut sender_lock = SENDER.borrow(cs).borrow_mut();
            let mut sender = sender_lock.take().expect("STRANGE");
            let packet = sender.get_next_packet_to_send();
            sender_lock.replace(sender);
            packet
        });
        match next_packet {
            Ok(Some(raw_packet)) => cortex_m::interrupt::free(|cs| {
                let mut lock = CAN.borrow(cs).borrow_mut();
                let mut can = lock.take().expect("honooo can");
                block!(can.transmit(&Frame::new_data(StandardId::new(1u16).unwrap(), raw_packet)));
                lock.replace(can);
            }),
            Ok(None) => {
                hprintln!("No packet to send");
            }
            Err(e) => {
                hprintln!("Unrecoverable error : {:?}", e);
                panic!();
            }
        }
    }
}
