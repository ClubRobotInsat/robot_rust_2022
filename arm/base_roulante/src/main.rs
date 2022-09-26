#![no_std]
#![no_main]

use core::convert::TryInto;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicI32, Ordering};
use cortex_m_rt::entry;
use cortex_m_semihosting::{hprint, hprintln};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::{PwmPin, Qei};
use panic_semihosting as _;
use stm32f1::stm32f103::interrupt;
use stm32f1::stm32f103::Interrupt;
use stm32f1::stm32f103::NVIC;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::qei::QeiOptions;
use stm32f1xx_hal::timer::{Event, Timer};
use stm32f1xx_hal::{pac, qei};
use stm32f1xx_hal::gpio::{CRH, Output, Pin, PushPull};

const MAX_SYST_VALUE: u32 = 0x00ffffff;

// static mut QEIL: MaybeUninit<stm32f1xx_hal::qei::Qei<stm32f1xx_hal::pac::TIM4, Tim4NoRemap, (stm32f1xx_hal::gpio::Pin<Input<Floating>, CRL, 'B', 6_u8>, stm32f1xx_hal::gpio::Pin<Input<Floating>, CRL, 'B', 7_u8>)>> =  MaybeUninit::uninit();
static mut TIM4_P: MaybeUninit<&mut Timer<stm32f1xx_hal::pac::TIM4>> = MaybeUninit::uninit();

struct Motor<DIR: OutputPin, PWM: PwmPin<Duty = u16>, CW: Qei<Count = u16>> {
    direction: DIR,
    pwm: PWM,
    coding_wheel: CW,
    last_value: u16,
    corrector: PID,
    encoder_sync:bool,
    orientation: bool
}

impl<DIR: OutputPin, PWM: PwmPin<Duty = u16>, CW: Qei<Count = u16>> Motor<DIR, PWM, CW> {
    const MAX_CORRECTION: u64 = 1000;
    const MAX_DUTY_FACTOR: u16 = 50;

    fn new(direction: DIR, mut pwm: PWM, coding_wheel: CW, dir: bool, orientation: bool) -> Self {
        pwm.enable();
        if dir {
            Self {
                direction,
                pwm,
                coding_wheel,
                last_value: 0,
                corrector: PID::new(60, 10, 0, 10000),
                encoder_sync: dir,
                orientation
            }
        }
        else {
            Self {
                direction,
                pwm,
                coding_wheel,
                last_value: 0,
                corrector: PID::new(60, 10, 0, -10000),
                encoder_sync: dir,
                orientation
            }
        }
    }

    pub fn update(&mut self, offset: i32) {
        // dt = time delta between calls
        // dt en ticks (= 1µs)
        let mut value: i32 = self.coding_wheel.count() as i32 + offset;
        if self.orientation {
            value = -value;
        }

        let error: i32 = self.corrector.target as i32 - (value as i32);

        //hprintln!("{:?} {:?} {:?}", LEFT_OFFSET, value, self.coding_wheel.count());
        if error > 0 {
            if self.encoder_sync {
                self.direction.set_high().ok();
            } else {
                self.direction.set_low().ok();
            }
        } else {
            if self.encoder_sync {
                self.direction.set_low().ok();
            } else {
                self.direction.set_high().ok();
            }
        }

        if error.unsigned_abs() < self.corrector.margin as u32  {
            self.pwm.set_duty(0);
            return;
        }

        let error: u16 = error.unsigned_abs() as u16;
        let proportional =  self.corrector.kp as u32 * error as u32;
        let integral = self.corrector.ki as u32 * error as u32;
        self.corrector.integral += integral;
        let integral = self.corrector.integral;
        let mut pid_correction = (proportional + integral )  / 100;
        pid_correction = pid_correction.min(Self::MAX_CORRECTION as u32);

        let max_duty = (self.pwm.get_max_duty() * Self::MAX_DUTY_FACTOR) / 100;
        let duty = (max_duty as u32 * pid_correction) /Self::MAX_CORRECTION as u32;
        self.pwm.set_duty(duty.try_into().expect("duty trop grand"));
        // hprintln!("enc: {}", value);
        // hprintln!("duty: {}, max: {}", duty, max_duty);
        // hprintln!("err: {}", error);


        // let corrected_setpoint = (pid_correction * max_duty as u64 / Self::MAX_CORRECTION);
        //
        // self.pwm.set_duty(corrected_setpoint as u16);
        //
        // self.last_value = value;
    }
}

struct PID {
    kp: u16,
    ki: u16,
    kd: u16,
    last_error: u16,
    integral: u32,
    target: i32,
    margin: u16
}

impl PID {
    fn new(kp: u16, ki: u16, kd: u16, consigne: i32) -> Self {
        Self {
            kp,
            ki,
            kd,
            last_error: 0,
            integral: 0,
            target: consigne,
            margin: 200
        }
    }

    // fn compute(&mut self, error: f32, dt: f32) -> f32 {
    //     self.integral += error;
    //     let d_err = (self.last_error - error) / dt;
    //     error * self.kp + self.integral * self.ki + d_err * self.kd
    // }
}

unsafe fn clear_tim4interrupt_bit() {
    (*stm32f1::stm32f103::TIM4::ptr())
        .sr
        .write(|w| w.uif().clear_bit());
}
unsafe fn clear_tim3interrupt_bit() {
    (*stm32f1::stm32f103::TIM3::ptr())
        .sr
        .write(|w| w.uif().clear_bit());
}

static LEFT_OFFSET: AtomicI32 = AtomicI32::new(0);
static RIGHT_OFFSET: AtomicI32 = AtomicI32::new(0);

#[interrupt]
fn TIM4() {
    let count = unsafe {
        (*stm32f1::stm32f103::TIM4::ptr())
            .cnt.read().bits()
    };

    if count < 65535/2 {
        LEFT_OFFSET.fetch_add(65536, Ordering::Relaxed);
    } else {
        LEFT_OFFSET.fetch_sub(65536, Ordering::Relaxed);
    }
    unsafe { clear_tim4interrupt_bit() }
}
#[interrupt]
fn TIM3() {
    let count = unsafe {
        (*stm32f1::stm32f103::TIM3::ptr())
            .cnt.read().bits()
    };
    if count < 65535/2 {
        RIGHT_OFFSET.fetch_add(65536, Ordering::Relaxed);
    } else {
        RIGHT_OFFSET.fetch_sub(65536, Ordering::Relaxed);
    }

    unsafe { clear_tim3interrupt_bit() }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    cp.SYST.set_reload(MAX_SYST_VALUE);
    cp.SYST.clear_current();
    cp.SYST.enable_counter();
    let counter = cp.SYST.counter_us(&clocks);

    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();

    let left_pwm_pin = gpioa.pa8.into_alternate_push_pull(&mut gpioa.crh);
    let right_pwm_pin = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);

    let pwm_timer = Timer::new(dp.TIM1, &clocks).pwm_hz(
        (left_pwm_pin, right_pwm_pin),
        &mut afio.mapr,
        50.kHz(),
    );
    let (mut pwm_left, mut pwm_right) = pwm_timer.split();

    let left_coding_a = gpiob.pb6.into_floating_input(&mut gpiob.crl);
    let left_coding_b = gpiob.pb7.into_floating_input(&mut gpiob.crl);
    let mut dir_left = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // hprintln!("coucou");
    let mut tim4 = Timer::new(dp.TIM4, &clocks);
    tim4.listen(Event::Update);
    let qei_left = tim4.qei(
        (left_coding_a, left_coding_b),
        &mut afio.mapr,
        QeiOptions {
            slave_mode: qei::SlaveMode::EncoderMode1,
            auto_reload_value: 65535,
        },
    );
    let right_coding_a = gpioa.pa6.into_floating_input(&mut gpioa.crl);
    let right_coding_b = gpioa.pa7.into_floating_input(&mut gpioa.crl);

let mut tim3 = Timer::new(dp.TIM3, &clocks);
    tim3.listen(Event::Update);
    let qei_right = tim3.qei(
        (right_coding_a, right_coding_b),
        &mut afio.mapr,
        QeiOptions {
            slave_mode: qei::SlaveMode::EncoderMode1,
            auto_reload_value: 65535,
        },
    );

    let mut left_motor = Motor::new(
        gpiob.pb12.into_push_pull_output(&mut gpiob.crh),
        pwm_left,
        qei_left,
        false,
        false
    );
    let mut right_motor = Motor::new(
        gpiob.pb13.into_push_pull_output(&mut gpiob.crh),
        pwm_right,
        qei_right,
        true,
        true,
    );

    let mut timer = Timer::new(dp.TIM2, &clocks).counter_hz();
    timer.start(5.Hz()).unwrap();

    // dp.TIM4.listen_interrupt() ça fat quoi???
    // enable interrupt
    unsafe { NVIC::unmask(Interrupt::TIM4) }
    unsafe { NVIC::unmask(Interrupt::TIM3) }
    dir_left.set_high();
    // dir_left.set_low();
    // hprint!("cucou");
    LEFT_OFFSET.store(0, Ordering::Relaxed);
    RIGHT_OFFSET.store(0, Ordering::Relaxed);
    loop {
        //block!(timer.wait());
        left_motor.update(LEFT_OFFSET.load(Ordering::Relaxed));
         right_motor.update(RIGHT_OFFSET.load(Ordering::Relaxed));
    }
}
