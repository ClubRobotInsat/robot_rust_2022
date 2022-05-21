#![no_std]
#![no_main]

// use core::ptr::read;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use drs_0x01::Rotation::Clockwise; // The runtime
                                   //use embedded_hal::digital::v2::OutputPin; // the `set_high/low`function

#[allow(unused_imports)]
use panic_halt;
use stm32f1xx_hal::pac::Peripherals;
use stm32f1xx_hal::prelude::*; // STM32F1 specific functions
use stm32f1xx_hal::rcc::Rcc;
use stm32f1xx_hal::rcc::RccExt;

extern crate drs_0x01;

use stm32f1xx_hal::{
    pac,
    serial::{Config, Serial},
};

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
    let dp: Peripherals = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    // GPIO pins on the STM32F1 must be driven by the APB2 peripheral clock.
    // This must be enabled first. The HAL provides some abstractions for

    // us: First get a handle to the RCC peripheral:
    let rcc: Rcc = dp.RCC.constrain();
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
    let pin_rx = gpioa.pa10;

    let serial = Serial::usart1(
        dp.USART1,
        (pin_tx, pin_rx),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()), // baud rate defined in herkulex doc : 115200
        clocks_serial.clone(),
        // &mut rcc.apb2,
    );

    // separate into tx and rx channels
    let (mut tx, rx) = serial.split();
    let mut delay = cp.SYST.delay(&clocks_serial);

    hprintln!("Debut");
    let communication = Communication::new(&mut tx, rx);
    hprintln!("Comm cree");

    let motors = Motors::new(communication);
    hprintln!("Motors cree");

    let group1 = motors.new_group();
    hprintln!("1er grp cree");

    let group2 = motors.new_group();
    hprintln!("groupes crees");

    let motor0 = group1.new_motor(0x00);
    let motor1 = group1.new_motor(0x01);
    let motor2 = group2.new_motor(0x02);
    let motor3 = group2.new_motor(0x03);

    hprintln!("Creation motors et groups");

    motor0.reboot();
    motor1.reboot();
    delay.delay_ms(250_u16);
    motor0.clear_errors();
    motor1.clear_errors();
    delay.delay_ms(100_u16);
    motor0.enable_torque();
    motor1.enable_torque();
    delay.delay_ms(100_u16);

    hprintln!("Init fini");

    loop {
        hprintln!("Boucle");

        let t = motor0.get_temperature();
        hprintln!("{:?}", t);
        delay.delay_ms(100_u16);

        motor0.set_speed(512, Clockwise);
        motor1.set_speed(0, Clockwise);
        delay.delay_ms(500_u16);

        motor0.set_speed(0, Clockwise);
        motor1.set_speed(512, Clockwise);
        delay.delay_ms(500_u16);
    }
}
