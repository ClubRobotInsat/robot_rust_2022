#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m::asm::delay;
#[allow(unused_imports)]
use panic_halt;

use cortex_m_semihosting::hprintln;

use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*,
                    qei::{QeiOptions, SlaveMode},
                    timer,
                    delay::Delay,
                    pwm::Channel,
                    time::U32Ext,
                    timer::{Tim2NoRemap, Timer},};

use panic_halt as _;

use cortex_m::asm;
use stm32f1xx_hal::gpio::{Alternate, PushPull};
use stm32f1xx_hal::pwm::Pwm;
use stm32f1xx_hal::pwm_input::Configuration::DutyCycle;

#[entry]
fn main() -> ! {
    //initiation
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain();

    let gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    let (_pa15, _pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

    // TIM3
    let p0 = pb4.into_alternate_push_pull(&mut gpiob.crl);
    let p1= gpiob.pb5.into_alternate_push_pull(&mut gpiob.crl);

    let pwm = Timer::tim3(dp.TIM3, &clocks, &mut rcc.apb1).pwm((p0, p1), &mut afio.mapr, 1.khz());

    let max = pwm.get_max_duty();

    let mut pwm_channels = pwm.split();

    // Enable the individual channels
    pwm_channels.0.enable();
    pwm_channels.1.enable();

    //pwm_channels.0.set_duty(70_u16);
    //pwm_channels.1.set_duty(max/2);

    // TIM4
    let c1 = gpiob.pb6;
    let c2 = gpiob.pb7;

    //initiation encoder mode 3
    let QeiOptions = QeiOptions{ slave_mode: SlaveMode::EncoderMode3, auto_reload_value: 65535 };
    let qei = Timer::tim4(dp.TIM4, &clocks, &mut rcc.apb1).qei((c1, c2), &mut afio.mapr, QeiOptions);
    let mut delay = Delay::new(cp.SYST, clocks);

    //PID
    //parameters PID
    let k = 1.0; //gain
    let ti = 1.0; //integration constant
    let td = 1.0; //derived constant

    let accept_error = 10.0;//1cm;

    //initiation PID
    let mut setpoint = 1000.0; //ask it to the user
    let mut measured_value = 0.0;
    let mut previous_value : f32 = ((qei.count()*2) as f32)/10.0; //get it from the captor in mm

    let mut error:f32 = 0.0;
    let dt:f32 = 0.001;
    let mut integration :f32 = 0.0; //integrate component
    let mut derived :f32 = 0.0;//derived component;
    let mut pid_correction:f32 = 0.0;


    loop{
        //pwm_channels.0.set_duty(3*max/4);
        //pwm_channels.1.set_duty(max);


        while error>accept_error {
            delay.delay_ms(1_u16);
            //PID operations
            //get informations
            measured_value = ((qei.count()*2) as f32)/10.0; //get it from the captor in mm;
            //time = timer::counter::SysCounter.now();  //get it from the timer

            error = setpoint - measured_value;

            //operations
            integration = integration + (&error)*dt;
            derived = (&error - &previous_value) / dt;

            pid_correction = k*error + ti*integration + td*derived; //value you can use for next step of the system

            //keep informations for the next round
            //PID informations
            previous_value = measured_value;
            //previous_time = time;

            match pid_correction {
                0.0..=1.0 => {
                    pwm_channels.0.set_duty(max*(pid_correction as u16));
                    pwm_channels.1.set_duty(max*(pid_correction as u16));
                },
                _ => {
                    pwm_channels.0.set_duty(max/2);
                    pwm_channels.1.set_duty(max/2);
                },
            }
        }

    }

}
