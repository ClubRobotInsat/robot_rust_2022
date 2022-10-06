//! Simple CAN example.
//! Requires a transceiver connected to PA11, PA12 (CAN1) or PB5 PB6 (CAN2).

#![no_main]
#![no_std]

use core::borrow::BorrowMut;
use core::cell::RefCell;
// use core::mem::MaybeUninit;
use panic_halt as _;
use drs_0x01::Rotation::Clockwise;

use crate::pac::NVIC;
//use crate::protocol::Message;
use bxcan::filter::Mask32;
use bxcan::Interrupt::Fifo0MessagePending;
use bxcan::{Frame, StandardId};
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use heapless::Vec;
use nb::block;
//use network_protocol;
//use network_protocol::MessageSender;
use stm32f1::stm32f103::{Interrupt, CAN1};
use stm32f1xx_hal::can::Can;
use stm32f1xx_hal::gpio::{Alternate, Floating, Input, Pin, PushPull, CRH};
use stm32f1xx_hal::timer::Timer;
use stm32f1xx_hal::{
    pac::{self, interrupt},
    prelude::*,
};
extern crate drs_0x01;
extern crate herkulex_drs_0x01_stm32f1xx;

use stm32f1xx_hal::{
    serial::{Config, Serial},
};

use herkulex_drs_0x01_stm32f1xx::*;
use herkulex_drs_0x01_stm32f1xx::communication::Communication;
use herkulex_drs_0x01_stm32f1xx::motors::Motors;

const ID: u8 = 2;

// type bxcan::Can<Can<CAN1>>> not to be confused with the totally different type stm32f1xx_hal::Can<Can<CAN1>>>
static CAN: Mutex<RefCell<Option<bxcan::Can<Can<CAN1>>>>> = Mutex::new(RefCell::new(None));

//global read variable
static READ: Mutex<RefCell<Vec<u8,8>>> = Mutex::new(RefCell::new(Vec::new()));

#[interrupt]
fn USB_LP_CAN_RX0() {
    cortex_m::interrupt::free(|cs| {
        // We need ownership of can ( bc it doesnt work without it ) so we take ownership out of the Option replacing it with None in the mutex and at the end of the critical section we replace the
        let mut mutex_lock = CAN.borrow(cs).borrow_mut();
        let mut can = mutex_lock.take().unwrap();
        // we always have a value as NVIC enable interrupt is after the blocking can setup
        match block!(can.receive()) {
            Ok(v) => unsafe {
                //hprintln!("Read");
                let read = v.data().unwrap().as_ref();
                let mutex_read_lock = READ
                    .borrow(cs)
                    .replace(
                        Vec::from_slice(read)
                            .unwrap()
                    );

                //Check ID = 1
                //hprintln!("ID = {:?}", v.data().unwrap());
                // for i in read {
                //     hprintln!("{}", i);
                // }
            },
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

    let mut gpioa = dp.GPIOA.split();

    let mut can1 = {
        let can: stm32f1xx_hal::can::Can<CAN1> = stm32f1xx_hal::can::Can::new(dp.CAN1, dp.USB);
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

    // Split the peripheral into transmitter and receiver parts.
    block!(can.enable_non_blocking()).unwrap();
    can.enable_interrupt(Fifo0MessagePending);

    cortex_m::interrupt::free(|cs| CAN.borrow(cs).replace(Some(can)));

    unsafe { NVIC::unmask(Interrupt::USB_LP_CAN_RX0) }

    // Echo back received packages in sequence.
    // See the `can-rtfm` example for an echo implementation that adheres to
    // correct frame ordering based on the transfer id.

    //New Delay with timer
    //let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    //timer.start(1.Hz()).unwrap();

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

    // USART1 on Pins A9 and A10
    let pin_tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);

    let pin_rx = gpioa.pa10;

    let serial = Serial::usart1(
        dp.USART1,
        (pin_tx, pin_rx),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()), // baud rate defined in herkulex doc : 115200
        clocks.clone(),
        // &mut rcc.apb2,
    );

    // separate into tx and rx channels
    let (mut tx, rx) = serial.split();
    let mut delay = cp.SYST.delay(&clocks);

    hprintln!("Communication créée").ok();
    hprintln!("Communication créée").ok();


    // La communication est une communication USART utilisant 2 ports : RX et TX
    // - TX est le port utilisé pour transmettre des informations
    // - RX est le port utilise pour recevoir des informations
    let communication = Communication::new(&mut tx, rx);
    hprintln!("Communication créée");

    let motors = Motors::new(communication);
    hprintln!("Motors créé");

    let motor2 = motors.new_motor(0x02);

    motor2.reboot();
    // hprintln!("servos redémarrés");
    delay.delay_ms(400_u16);

    // Afin de faire tourner les servos il est nécessaire d'activer le torque
    // Dans le cas contraire les ser
    motor2.enable_torque();
    // hprintln!("servos torque activé");
    delay.delay_ms(100_u16);


    hprintln!("Debut");

    loop {
        let mut cmd : Vec<u8,8> = Vec::new();
        cortex_m::interrupt::free(|cs | {
            cmd = READ.borrow(cs).replace(Vec::new());
        });

        match cmd.last(){
            None => {
                motor2.set_speed(0, Clockwise);
                //hprintln!("None");
                delay.delay_ms(2000_u16);
            }
            Some(v) => {
                if v == &1_u8{
                    motor2.set_speed(512, Clockwise);
                    //hprintln!("tourne");
                    delay.delay_ms(2000_u16);
                } else {
                    motor2.set_speed(0, Clockwise);
                    //hprintln!("arrete");
                    delay.delay_ms(2000_u16);
                }
            }
        }
        //block!(timer.wait()).unwrap();

        //Send CODE
        /*
               block!(can.transmit(&_data1)).unwrap();

               block!(timer.wait()).unwrap();
               block!(can.transmit(&_data2)).unwrap();
               //Wait 1 second
               block!(timer.wait()).unwrap();

               block!(can.transmit(&_data_off1)).unwrap();
               block!(timer.wait()).unwrap();
               block!(can.transmit(&_data_off2)).unwrap();
               //Wait 1 second
               block!(timer.wait()).unwrap();

        */
    }
}

