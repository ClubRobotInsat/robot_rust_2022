//! Simple CAN example.
//! Requires a transceiver connected to PA11, PA12 (CAN1) or PB5 PB6 (CAN2).

#![no_main]
#![no_std]


use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::mem::MaybeUninit;
use panic_halt as _;

use bxcan::filter::Mask32;
use bxcan::{Frame, StandardId};
use bxcan::Interrupt::Fifo0MessagePending;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nb::block;
use stm32f1xx_hal::{pac::{self,interrupt}, prelude::*};
use stm32f1xx_hal::timer::Timer;
use crate::pac::NVIC;
use stm32f1::stm32f103::{CAN1, Interrupt};
use stm32f1xx_hal::can::Can;
use network_protocol;
use network_protocol::MessageSender;
use stm32f1xx_hal::gpio::{CRH, Floating, Input, Pin};

// type bxcan::Can<Can<CAN1>>> not to be confused with the totally different type stm32f1xx_hal::::Can<Can<CAN1>>>
static CAN : Mutex<RefCell<Option<bxcan::Can<Can<CAN1>>>>> = Mutex::new(RefCell::new(None));
static sender: Option<MessageSender<Tx, Rx>> = None;


#[interrupt]
fn USB_LP_CAN_RX0() {
    cortex_m::interrupt::free(|cs| {
        // We need ownership of can ( bc it doesnt work without it ) so we take ownership out of the Option replacing it with None in the mutex and at the end of the critical section we replace the
        let mut mutex_lock = CAN.borrow(cs).borrow_mut();
        let mut can = mutex_lock
            .take()
            .unwrap();
        // we always have a value as NVIC enable interrupt is after the blocking can setup
        match block!(can.receive()) {
            Ok(v) => {
                //hprintln!("Read");
                let read = v.data().unwrap().as_ref();
                //Check ID = 1
                //hprintln!("ID = {:?}", v.data().unwrap());
                if read[0] == 2 {
                    if read[2] == 1 {
                        hprintln!("HIGH");
                    } else if read[2] == 0 {
                        hprintln!("LOW");
                    } else {
                        hprintln!("Unknown Command");
                    }
                } else {
                    hprintln!("NOT ME");
                }
            }
            Err(e) => {
                hprintln!("err",);
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
        let can: stm32f1xx_hal::can::Can::<CAN1> = stm32f1xx_hal::can::Can::new(dp.CAN1, dp.USB);

        let mut gpioa = dp.GPIOA.split();
        let rx = gpioa.pa11.into_floating_input(&mut gpioa.crh);
        let tx = gpioa.pa12.into_alternate_push_pull(&mut gpioa.crh);
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
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // Split the peripheral into transmitter and receiver parts.
    block!(can.enable_non_blocking()).unwrap();
    can.enable_interrupt(Fifo0MessagePending);

    cortex_m::interrupt::free(|cs| CAN.borrow(cs).replace(Some(can)));

    unsafe {NVIC::unmask(Interrupt::USB_LP_CAN_RX0)}

    // Echo back received packages in sequence.
    // See the `can-rtfm` example for an echo implementation that adheres to
    // correct frame ordering based on the transfer id.

    //New Delay with timer
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();

    //Send data
    let data1 = Frame::new_data(StandardId::new(1_u16).unwrap(),[1_u8, 1_u8 ,1_u8, 1_u8, 1_u8, 1_u8, 1_u8, 1_u8]);
    let data_off1 = Frame::new_data(StandardId::new(1_u16).unwrap(),[1_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8]);
    let data2 = Frame::new_data(StandardId::new(1_u16).unwrap(),[2_u8, 1_u8 ,1_u8, 1_u8, 1_u8, 1_u8, 1_u8, 1_u8]);
    let data_off2 = Frame::new_data(StandardId::new(1_16).unwrap(), [2_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8]);

    hprintln!("Debut");

    loop{

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
