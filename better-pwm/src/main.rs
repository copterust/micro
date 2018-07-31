#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(used)]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate panic_abort;

extern crate alt_stm32f30x_hal as hal;
extern crate stm32f30x;

use hal::gpio;
use hal::gpio::gpiob::PB8;
use hal::prelude::*;
use hal::pwm::PwmBinding;
use hal::timer;
use hal::timer::tim4;

use rt::ExceptionFrame;

use cortex_m::asm;

entry!(main);
fn main() -> ! {
    let device = hal::stm32f30x::Peripherals::take().unwrap();
    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.constrain();

    let gpiob = device.GPIOB.split(&mut rcc.ahb);
    // get port b
    let pb8 = gpiob.pb8.pull_type(gpio::PullUp);
    let clocks = rcc
        .cfgr
        .sysclk(64.mhz())
        .pclk1(32.mhz())
        .freeze(&mut flash.acr);
    let tim4 = timer::tim4::Timer::new(device.TIM4, 1.mhz(), clocks, &mut rcc.apb1);
    let (ch3, mut tim4) = tim4.take_ch3();
    // Two ways to create binding: via named func or via turbo fishing:
    // let mut pwm = PwmBinding::bind_pb6_tim4_ch1(pb6, ch1);
    let mut pwm = PwmBinding::<PB8<_, _>, tim4::Channel<timer::CH3, _>, gpio::AF2>::new(pb8, ch3);
    pwm.enable();
    tim4.enable();

    loop {
        for i in 10..500 {
            pwm.set_duty(i);
            tick_delay(2500);
        }
        for i in 10..500 {
            pwm.set_duty(500 - i);
            tick_delay(2500);
        }
    }
}

exception!(HardFault, hard_fault);
fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);
fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

fn tick_delay(ticks: usize) {
    (0..ticks).for_each(|_| asm::nop());
}
