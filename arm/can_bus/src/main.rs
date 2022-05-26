//! Simple CAN example.
//! Requires a transceiver connected to PA11, PA12 (CAN1) or PB5 PB6 (CAN2).

#![no_main]
#![no_std]

use core::cell::RefCell;
use panic_halt as _;

use crate::pac::NVIC;
use crate::protocol::header::Header;
use crate::protocol::message::Message;
use crate::protocol::message_in_progress::Messages;
use crate::protocol::packet::Packet;
use crate::protocol::{CanId, MessageId, SeqId};
use bxcan::filter::Mask32;
use bxcan::Interrupt::Fifo0MessagePending;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use heapless::Vec;
use nb::block;
use stm32f1::stm32f103::{Interrupt, CAN1};
use stm32f1xx_hal::can::Can;
use stm32f1xx_hal::timer::Timer;
use stm32f1xx_hal::{
    pac::{self, interrupt},
    prelude::*,
};

mod protocol;

const HOST_ID: u8 = 3;

static CAN: Mutex<RefCell<Option<bxcan::Can<Can<CAN1>>>>> = Mutex::new(RefCell::new(None));
static SENDER: Mutex<RefCell<Option<Messages>>> = Mutex::new(RefCell::new(None));

#[allow(non_snake_case)]
#[interrupt]
fn USB_LP_CAN_RX0() {
    cortex_m::interrupt::free(|cs| {
        // We need ownership of can ( bc it doesnt work without it ) so we take ownership out of the Option replacing it with None in the mutex and at the end of the critical section we replace the
        let mut mutex_lock = CAN.borrow(cs).borrow_mut();
        let mut can = mutex_lock.take().unwrap();
        // we always have a value as NVIC enable interrupt is after the blocking can setup
        match block!(can.receive()) {
            Ok(v) => {
                //hprintln!("Read");
                let read: [u8; 8] = <[u8; 8]>::try_from(v.data().unwrap().as_ref()).unwrap();
                SENDER.borrow(&cs).take().unwrap().process_raw_packet(read);
                //Check ID = 1
                //hprintln!("ID = {:?}", v.data().unwrap());
            }
            Err(e) => {
                hprintln!("err: {:?}", e).ok();
            }
        }
        // we always have a value as NVIC enable interrupt is after the blocking can setup
        match can.receive() {
            Ok(_) => {
                // TODO: Traiter la réception
                // let read = frame.data().unwrap().as_ref();
            }
            Err(e) => {
                hprintln!("err: {:?}", e).ok();
            }
        }
        mutex_lock.replace(can);
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
    let mut gpioc = dp.GPIOC.split();

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
            Messages::new(CanId::new(HOST_ID as usize).unwrap()).unwrap(),
        ));
    });

    unsafe { NVIC::unmask(Interrupt::USB_LP_CAN_RX0) }

    // New Delay with timer
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();

    hprintln!("Début de l'envoi");

    let mut sender1 = Messages::new(CanId::new(2).unwrap()).unwrap();

    sender1.add_message_to_send_buff(
        Message::new(
            MessageId::new(5).unwrap(),
            CanId::new(3).unwrap(),
            sender1.host_id,
            Vec::from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap(),
        )
        .unwrap(),
    );

    loop {
        block!(timer.wait()).unwrap();

        hprintln!(
            "Next packet: {:?}",
            sender1
                .get_next_packet_to_send()
                .unwrap_or(Some(Vec::<u8, 8>::from_slice(&[0; 8]).unwrap()))
                .unwrap(),
        )
        .ok();

        sender1
            .process_raw_packet(
                Packet::new(
                    Header::new(
                        sender1.host_id,
                        CanId::new(5).unwrap(),
                        false,
                        MessageId::new(3).unwrap(),
                        SeqId::new(0).unwrap(),
                    )
                    .unwrap(),
                    [9, 10, 11, 12, 13, 14],
                )
                .get_packet_as_binary_array(),
            )
            .unwrap();
        hprintln!("rec: {:?}", sender1.received.clone()).ok();
        //Send CODE
        /*
                    block!(can.transmit(&data1)).unwrap();

                    block!(timer.wait()).unwrap();
                    block!(can.transmit(&data2)).unwrap();
                    //Wait 1 second
                    block!(timer.wait()).unwrap();

                    block!(can.transmit(&data_off1)).unwrap();
                    block!(timer.wait()).unwrap();
                    block!(can.transmit(&data_off2)).unwrap();
                    //Wait 1 second
                    block!(timer.wait()).unwrap();
        */
        //Receive CODE
        //ID recognition

        //
        //     match block!(can.receive()) {
        //         Ok(v) => {
        //             //hprintln!("Read");
        //             let read = v.data().unwrap().as_ref();
        //             //Check ID = 1
        //             //hprintln!("ID = {:?}", v.data().unwrap());
        //             if read[0] == 2{
        //
        //                 if  read[2] == 1 {
        //                     led.set_high();
        //                     hprintln!("HIGH");
        //                 }
        //                 else if read[2] == 0 {
        //                     led.set_low();
        //                     hprintln!("LOW");
        //                 } else {
        //                     hprintln!("Unknown Command");
        //                 }
        //             } else {
        //                 hprintln!("NOT ME");
        //             }
        //
        //         }
        //         Err(e) => {
        //             hprintln!("err",);
        //         }
        //     }
        //
    }
}
