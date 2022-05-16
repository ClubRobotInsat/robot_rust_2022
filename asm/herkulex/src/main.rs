#![no_std]
#![no_main]

use core::convert::Infallible;

// use core::ptr::read;
use cortex_m_rt::entry; // The runtime
//use embedded_hal::digital::v2::OutputPin; // the `set_high/low`function
use stm32f1xx_hal::{  prelude::*}; // STM32F1 specific functions
#[allow(unused_imports)]
use panic_halt;
use stm32f1xx_hal::rcc::Rcc;
use stm32f1xx_hal::rcc::RccExt;
use stm32f1xx_hal::pac::Peripherals;
use cortex_m::{ singleton};

use nb::{block, Error};
use cortex_m_semihosting::{hprint, hprintln};
extern crate drs_0x01;
use drs_0x01::{Servo, Rotation, JogMode, JogColor, WritableEEPAddr, ReadableEEPAddr, WritableRamAddr, ReadableRamAddr};
use drs_0x01::builder::{HerkulexMessage, MessageBuilder};
use drs_0x01::reader::ACKReader;
use stm32f1xx_hal::timer::delay::Delay;
use stm32f1xx_hal::{
    pac,
    pac::interrupt,
    pac::USART1,
    prelude::*,
    gpio::*,
    serial::{self, Config, Serial, Event, Rx, Tx},
};
use core::mem::MaybeUninit;
use drs_0x01::Rotation::Clockwise;
use stm32f1xx_hal::serial::RxDma1;

use unwrap_infallible::UnwrapInfallible;
use crate::motors::communication::Communication;
use crate::motors::Motors;

pub mod motors;

/// a functuin
pub fn a() {}

#[entry]
fn main() -> ! {
    // Get handles to the hardware objects. These functions can only be called
    // once, so that the borrowchecker can ensure you don't reconfigure
    // something by accident.
    let mut dp: Peripherals = pac::Peripherals::take().unwrap();

    // GPIO pins on the STM32F1 must be driven by the APB2 peripheral clock.
    // This must be enabled first. The HAL provides some abstractions for

    // us: First get a handle to the RCC peripheral:
    let mut rcc: Rcc = dp.RCC.constrain();
    // Now we have access to the RCC's registers. The GPIOC can be enabled in
    // RCC_APB2ENR (Prog. Ref. Manual 8.3.7), therefore we must pass this
    // register to the `split` function.
    // This gives us an exclusive handle to the GPIOC peripheral. To get the
    // handle to a single pin, we need to configure the pin first. Pin C13
    // is usually connected to the Bluepills onboard LED.

    let mut flash = dp.FLASH.constrain();
    let mut gpioa = dp.GPIOA.split();
    let mut afio = dp.AFIO.constrain();

    // let sys_clock = rcc.cfgr.sysclk(8.mhz()).freeze(&mut flash.acr);
    let clocks_serial = rcc.cfgr.freeze(&mut flash.acr);

    // USART1 on Pins A9 and A10
    let pin_tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    // let pin_rx = gpioa.pa10;

    // let pin_rx = unsafe { &mut *RX_PIN.as_mut_ptr() };
    let mut pin_rx = gpioa.pa10;


    let serial = Serial::usart1(
        dp.USART1,
        (pin_tx, pin_rx),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()), // baud rate defined in herkulex doc : 115200
        clocks_serial.clone(),
        // &mut rcc.apb2,
    );

    // separate into tx and rx channels
    let (mut tx, mut rx) = serial.split();

    let communication = Communication::new(&mut tx, rx);
    let mut motors = Motors::new(communication);
    let mut group1 = motors.get_group(motors.new_group()).borrow_mut();
    let mut group2 = motors.get_group(motors.new_group()).borrow_mut();


    let _ = group1.as_ref().unwrap().new_motor(0x00);
    let _ = group1.as_ref().unwrap().new_motor(0x01);
    let _ = group2.as_ref().unwrap().new_motor(0x00);
    let _ = group2.as_ref().unwrap().new_motor(0x01);


    loop {

    }
}
