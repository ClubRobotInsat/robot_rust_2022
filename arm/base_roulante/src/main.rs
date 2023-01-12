#![no_std]
#![no_main]

use core::convert::TryInto;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicI32, Ordering};
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use cortex_m_semihosting::{hprint, hprintln};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::{PwmPin, Qei};

use panic_semihosting as _;
use stm32f1::stm32f103::{interrupt, TIM2};
use stm32f1::stm32f103::Interrupt;
use stm32f1::stm32f103::NVIC;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::qei::QeiOptions;
use stm32f1xx_hal::timer::{CounterHz, Event, Timer, TimerExt};
use stm32f1xx_hal::{pac, qei};
use stm32f1xx_hal::gpio::{CRH, Output, Pin, PushPull};
use stm32f1xx_hal::time::MicroSeconds;

const MAX_SYST_VALUE: u32 = 0x00ffffff;


// static mut QEIL: MaybeUninit<stm32f1xx_hal::qei::Qei<stm32f1xx_hal::pac::TIM4, Tim4NoRemap, (stm32f1xx_hal::gpio::Pin<Input<Floating>, CRL, 'B', 6_u8>, stm32f1xx_hal::gpio::Pin<Input<Floating>, CRL, 'B', 7_u8>)>> =  MaybeUninit::uninit();
static mut TIM4_P: MaybeUninit<&mut Timer<stm32f1xx_hal::pac::TIM4>> = MaybeUninit::uninit();
static motor_left:Mutex<Option<i32>>= Mutex::new(None);

//supongo que aqui define la estructura motor con diferentes valores.
//no entiendo el <>

struct Motor<DIR: OutputPin, PWM: PwmPin<Duty = u16>, CW: Qei<Count = u16>> {
    direction: DIR,
    pwm: PWM,
    coding_wheel: CW,
    last_value: u16,
    corrector: PID,
    encoder_sync:bool,
    orientation: bool,

}
//implementacion de la estructura motor, como lo cual tienes que especificar
// all again
//dentro de la impl creamos las funciones que queremos que pueda realizar el motor
impl<DIR: OutputPin, PWM: PwmPin<Duty = u16>, CW: Qei<Count = u16>> Motor<DIR, PWM, CW> {
    const MAX_CORRECTION: u64 = 1000;
    const MAX_DUTY_FACTOR: u16 = 50;

    fn new (direction: DIR, mut pwm: PWM, coding_wheel: CW, dir: bool, orientation: bool) -> Self {
        pwm.enable(); //enables a pwm channel, para ello hemos tenido que definir el pin pwm antes
        Self {
            direction,
            pwm,
            coding_wheel,
            last_value: 0,
            corrector: PID::new(1.0, 0.00001,0.0, if dir {1000} else {-1000}),
            encoder_sync: dir,
            orientation
        }
    }

    pub fn update(&mut self, offset: i32) {
        // dt = time delta between calls
        // dt en ticks (= 1µs)

        let encoder = self.coding_wheel.count() as i32;
        let mut value: i32 = encoder + offset;

        if self.orientation {
            value = -value;
        }

        let error: i32 = self.corrector.target as i32 - (value as i32);
        //let mut real_time= self.corrector.timer.now().ticks() ;

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


        let mut delta_t= 0;
        /*let time_max= 65535 as u32 ;
        if real_time<self.corrector.last_time {
            delta_t=(time_max-self.corrector.last_time+real_time);
        }
        else{
            delta_t=(real_time-self.corrector.last_time);
        } */


        let error: u32 = error.unsigned_abs() as u32;

        let proportional =  self.corrector.kp as u32 * error as u32;
        //let integral = (self.corrector.ki * (error as u32*delta_t) as f32) as f32;
        //self.corrector.integral += integral;
        //let integral = self.corrector.integral ;

        let  derivative =0;
        if (delta_t != 0) {
            ((self.corrector.kd as u32)* (error - (self.corrector.last_error as u32)) / delta_t)
        } else { 0 };
        //self.corrector.last_time=real_time; //creo que falta poner error en last value
        //let mut pid_correction = (proportional + integral + derivative ) ;
        let mut pid_correction = (proportional) ;
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
//lo que hacen las funciones de la impl MOTOR
//1. fn new: lo que haces es cambiar la consigne de 10000 o -10000
//2. fn update: supongo que lo hace es update el pid con la integral, derivador y el gain proporcional

/*pub fn update1(&mut self, offset: i32) {


    let encoder = self.coding_wheel.count() as i32;
    let mut value: i32 = encoder + offset;

    //traitement des possibles erreurs de branchement
    if self.orientation {
        value = -value;
    }

    //calcul de l'erreur
    let error: i32 = self.corrector.target as i32 - (value as i32);

    if error > 0 {
        if self.encoder_sync { //encodersyn : au cas ou l'encoder est connecté à l'envers du mouvement du moteur
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

    //une fois le signe de l'erreur a ete traire et les moteurs inversees si necessaire,
    //traitement de l'erreur en valeur absolue
    if error.unsigned_abs() < self.corrector.margin as u32  {
        self.pwm.set_duty(0);
        return;
    }

    //le delta t est donné par le timer2
    let mut delta_t= 0;
    let error: u32 = error.unsigned_abs() as u32;

    //CORRECTEUR PROPORTIONNEL
    let proportional =  self.corrector.kp as u32 * error as u32;

    //CORRECTEUR INTEGRAL
    let integral = (self.corrector.ki * (error as u32*delta_t) as f32) as f32;
    self.corrector.integral += integral;
    let integral = self.corrector.integral ;

    //CORRECTEUR DERIVATEUR
    let  derivative =((self.corrector.kd as u32)* (error - (self.corrector.last_error as u32)) / delta_t);

    //CORRECTEUR FINAL
    //let mut pid_correction = (proportional + integral + derivative ) ;
    let mut pid_correction = (proportional) ;
    pid_correction = pid_correction.min(Self::MAX_CORRECTION as u32);

    //SET DUTY MOTORS
    let max_duty = (self.pwm.get_max_duty() * Self::MAX_DUTY_FACTOR) / 100;
    let duty = (max_duty as u32 * pid_correction) /Self::MAX_CORRECTION as u32;
    self.pwm.set_duty(duty.try_into().expect("duty trop grand"));

}

*/

struct PID {
    kp: f32,
    ki: f32,
    kd: f32,
    last_error: u16,
    integral: f32,
    target: i32,
    margin: u16,
    last_time : u32,
    //timer: CounterHz<TIM2>,
}

//implementacion de las funciones para utilizar con PID
//1. creacion de una struct PID
impl PID {
    fn new(kp: f32, ki: f32, kd: f32, consigne: i32 ) -> Self {
        Self {
            kp,
            ki,
            kd,
            last_error: 0,
            integral: 0.0,
            target: consigne,
            margin: 200,
            last_time : 0 ,
            //timer,
        }
    }

    // fn compute(&mut self, error: f32, dt: f32) -> f32 {
    //     self.integral += error;
    //     let d_err = (self.last_error - error) / dt;
    //     error * self.kp + self.integral * self.ki + d_err * self.kd
    // }
}
//supongo que aqui configuran las interrupciones para la pwm del tim 3 y 4
//declaramos en un safe ara que no crash

//sirve para baisser el flag, lo utilizamos en el irq_handleer para que sea más visible.
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

unsafe fn clear_tim2interrupt_bit() {
    (*stm32f1::stm32f103::TIM2::ptr())
        .sr
        .write(|w| w.uif().clear_bit());
}

//el atomic sirve para decir que tant que j'utilise cette variable, no realizo ninguna interrupción
static LEFT_OFFSET: AtomicI32 = AtomicI32::new(0);
static RIGHT_OFFSET: AtomicI32 = AtomicI32::new(0);

#[interrupt]
fn TIM2() {
    cortex_m::interrupt::free(|cs| {
        let mut mutex_lock = motor_left.borrow(cs).borrow_mut();
        //desempaquetar porque el motor esta dentro del mutex
        let mut left_m = mutex_lock.take().unwrap();
        motor_left.update();
    });
    unsafe { clear_tim2interrupt_bit() }
}

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
    //cojo peri de la carte
    //cojo peri del cortex
    let dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

    //constraint-> ceder ownership
    let mut flash = dp.FLASH.constrain(); //tipo de peri (flash memory)
    let rcc = dp.RCC.constrain(); //
    let mut afio = dp.AFIO.constrain();
    //Clock configuration register cfgr
    //Used to configure the frequencies of the clocks present in the processor.
    //After setting all frequencies, call the freeze function to apply the configuration.
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    //SysTick: System Timer
    cp.SYST.set_reload(MAX_SYST_VALUE); //pone el valor en el timer
    cp.SYST.clear_current(); //Clears current value to 0
    cp.SYST.enable_counter(); //enable el timer?
    let counter = cp.SYST.counter_us(&clocks); //variable que te da nb del timer

    let mut gpioa = dp.GPIOA.split(); //split sirve para separar los gpio y poder tratarlos
    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();

    //creación de pin y configuración: pwm para cada motor de cada rueda supongo
    let left_pwm_pin = gpioa.pa8.into_alternate_push_pull(&mut gpioa.crh);
    let right_pwm_pin = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);

    //funcion para crear un timer
    let pwm_timer = Timer::new(dp.TIM1, &clocks).pwm_hz(
        (left_pwm_pin, right_pwm_pin),
        &mut afio.mapr,
        50.kHz(),
    );
    let (mut pwm_left, mut pwm_right) = pwm_timer.split(); //como se que da un tuple?

    //mirar a que corresponden pb6...
    let left_coding_a = gpiob.pb6.into_floating_input(&mut gpiob.crl);  //TIM4_CH1
    let left_coding_b = gpiob.pb7.into_floating_input(&mut gpiob.crl);  //TIM4_CH2
    let mut dir_left = gpioc.pc13.into_push_pull_output(&mut gpioc.crh); //parece el bit de sens

    //configuration des TIMERS:
    let mut tim4 = Timer::new(dp.TIM4, &clocks);
    tim4.listen(Event::Update);

    let mut tim2 = Timer::new(dp.TIM2, &clocks); //pour gerer les update du PID
    tim2.listen(Event::Update);

    //NI IDEA
    let qei_left = tim4.qei(
        (left_coding_a, left_coding_b),
        &mut afio.mapr,
        QeiOptions {
            slave_mode: qei::SlaveMode::EncoderMode1,
            auto_reload_value: 1024,
        },
    );

    //CONFIGURACION TIMER 3 PARA PWM
    let right_coding_a = gpioa.pa6.into_floating_input(&mut gpioa.crl); //TIM3_CH1
    let right_coding_b = gpioa.pa7.into_floating_input(&mut gpioa.crl); //TIM3_CH2


let mut tim3 = Timer::new(dp.TIM3, &clocks);
    tim3.listen(Event::Update);
    let qei_right = tim3.qei(
        (right_coding_a, right_coding_b),
        &mut afio.mapr,
        QeiOptions {
            slave_mode: qei::SlaveMode::EncoderMode1,
            auto_reload_value: 1024,
        },
    );

    //creacion left and right motor a partir de la struct MOTOR

    let mut left_motor = Motor::new(
        gpiob.pb12.into_push_pull_output(&mut gpiob.crh),
        pwm_left,
        qei_left,
        false,
        false,
        //timer2


    );
/*    let mut right_motor = Motor::new(
        gpiob.pb13.into_push_pull_output(&mut gpiob.crh),
        pwm_right,
        qei_right,
        true,
        true,
    );
*/


    // dp.TIM4.listen_interrupt() ça fat quoi???
    // enable interrupt

    //activo el NVIC  para el interrupt
    unsafe { NVIC::unmask(Interrupt::TIM4) }
    unsafe { NVIC::unmask(Interrupt::TIM3) }
    dir_left.set_high(); //NO SE QUE HACE
    // dir_left.set_low();
    // hprint!("cucou");
    LEFT_OFFSET.store(0, Ordering::Relaxed);
    RIGHT_OFFSET.store(0, Ordering::Relaxed);
    loop {
        //block!(timer.wait());
        left_motor.update(LEFT_OFFSET.load(Ordering::Relaxed));
         //right_motor.update(RIGHT_OFFSET.load(Ordering::Relaxed));
    }
}
