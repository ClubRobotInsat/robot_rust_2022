//! Simple CAN example.
//! Requires a transceiver connected to PA11, PA12 (CAN1) or PB5 PB6 (CAN2).

#![no_main]
#![no_std]

use core::borrow::BorrowMut;
use core::cell::RefCell;
// use core::mem::MaybeUninit;
use panic_halt as _;

use crate::pac::NVIC;
use crate::protocol::{Header, Message, Messages, Packet};
use bxcan::filter::Mask32;
use bxcan::Interrupt::Fifo0MessagePending;
use bxcan::{Frame, StandardId};
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use heapless::Vec;
use nb::block;
use stm32f1::stm32f103::{Interrupt, CAN1};
use stm32f1xx_hal::can::Can;
use stm32f1xx_hal::gpio::{Alternate, Floating, Input, Pin, PushPull, CRH};
use stm32f1xx_hal::timer::Timer;
use stm32f1xx_hal::{
    pac::{self, interrupt},
    prelude::*,
};

mod protocol;

const HOST_ID: u8 = 3;

// type bxcan::Can<Can<CAN1>>> not to be confused with the totally different type stm32f1xx_hal::::Can<Can<CAN1>>>
static CAN: Mutex<RefCell<Option<bxcan::Can<Can<CAN1>>>>> = Mutex::new(RefCell::new(None));
static SENDER: Mutex<RefCell<Option<Messages>>> = Mutex::new(RefCell::new(None));

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
                SENDER
                    .borrow(&cs)
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .process_message(read);
                //Check ID = 1
                //hprintln!("ID = {:?}", v.data().unwrap());
            }
            Err(e) => {
                hprintln!("err: {:?}", e).ok();
            }
        }
        mutex_lock.replace(can);
    })
}

#[entry]
//Symbol ! means the fonction returns NEVER => an infinite loop must exist
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    // To meet CAN clock accuracy requirements an external crystal or ceramic
    // resonator must be used. The blue pill has a 8MHz external crystal.
    // Other boards might have a crystal with another frequency or none at all.
    let clocks = rcc.cfgr.use_hse(8.MHz()).freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain();

    let mut can1 = {
        let can: stm32f1xx_hal::can::Can<CAN1> = stm32f1xx_hal::can::Can::new(dp.CAN1, dp.USB);

        let mut gpioa = dp.GPIOA.split();
        let rx: Pin<Input<Floating>, CRH, 'A', 11> = gpioa.pa11.into_floating_input(&mut gpioa.crh);
        let tx: Pin<Alternate<PushPull>, CRH, 'A', 12> =
            gpioa.pa12.into_alternate_push_pull(&mut gpioa.crh);
        can.assign_pins((tx, rx), &mut afio.mapr);

        bxcan::Can::builder(can)
            .set_bit_timing(0x001c_0003)
            .leave_disabled()
    };

    // APB1 (PCLK1): 8MHz, Bit rate: 125kBit/s, Sample Point 87.5%
    // Value was calculated with http://www.bittiming.can-wiki.info/
    can1.modify_config().set_bit_timing(0x001c_0003);

    // Configure filters so that can frames can be received.
    let mut filters = can1.modify_filters();
    filters.enable_bank(0, Mask32::accept_all());

    // Drop filters to leave filter configuraiton mode.
    drop(filters);

    // Select the interface.
    let mut can: bxcan::Can<Can<CAN1>> = can1;
    //let mut can = _can2;

    let mut gpioc = dp.GPIOC.split();
    let _led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // Split the peripheral into transmitter and receiver parts.
    block!(can.enable_non_blocking()).unwrap();
    can.enable_interrupt(Fifo0MessagePending);

    cortex_m::interrupt::free(|cs| {
        CAN.borrow(cs).replace(Some(can));
        SENDER
            .borrow(&cs)
            .replace(Some(Messages::new(HOST_ID).unwrap()));
    });

    unsafe { NVIC::unmask(Interrupt::USB_LP_CAN_RX0) }

    // Echo back received packages in sequence.
    // See the `can-rtfm` example for an echo implementation that adheres to
    // correct frame ordering based on the transfer id.

    //New Delay with timer
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();

    //Send data
    let _data1 = Frame::new_data(
        StandardId::new(1_u16).unwrap(),
        [1_u8, 1_u8, 1_u8, 1_u8, 1_u8, 1_u8, 1_u8, 1_u8],
    );
    let _data_off1 = Frame::new_data(
        StandardId::new(1_u16).unwrap(),
        [1_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8],
    );
    let _data2 = Frame::new_data(
        StandardId::new(1_u16).unwrap(),
        [2_u8, 1_u8, 1_u8, 1_u8, 1_u8, 1_u8, 1_u8, 1_u8],
    );
    let _data_off2 = Frame::new_data(
        StandardId::new(1_u16).unwrap(),
        [2_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8],
    );

    hprintln!("Debut");

    // let mut sender1 = protocol::Messages::new(2).unwrap();
    // sender1.add_message_to_send_buff(
    //     Message::new(5, 3, 2, Vec::from_slice(&[1, 2, 3, 4, 5, 6,7,8]).unwrap()).unwrap(),
    // );
    loop {
        block!(timer.wait()).unwrap();

        // hprintln!(
        //     "[]Next packet: {:?}",
        //     sender1
        //         .get_next_packet_to_send()
        //         .unwrap_or(Vec::<u8,8>::from_slice(&[0; 8]).unwrap()),
        //
        // )
        // .ok();
        // hprintln!("sending ack").ok();
        // sender1.process_message(Packet::new(Header::new(2,3,true,5,1).unwrap(),[0,0,0,0,0,0]).get_packet_as_binary_array());
        // sender1.respond_ack(Packet::new(Header::new(2,3,false,5,1).unwrap(),[0,0,0,0,0,0]));
        // for m in &sender1.send_buff {
        //     hprintln!("to send: {:?}", m.clone()).ok();
        // }
        // sender1.process_message(Packet::new(Header::new(sender1.host_id,5,false,3,0).unwrap(),[9,10,11,12,13,14]).get_packet_as_binary_array());
        // hprintln!("rec: {:?}", sender1.received.clone()).ok();
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
