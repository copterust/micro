#![deny(warnings)]
#![no_std]
#![no_main]

#[allow(unused)]
use panic_abort;

use core::fmt::{self, Write};

use hal::prelude::*;
use hal::time::Bps;
use hal::{delay, serial};
use nb;
use cortex_m_rt::{entry, exception, ExceptionFrame};

use mpu9250::Mpu9250;

#[entry]
fn main() -> ! {
    let device = hal::stm32f30x::Peripherals::take().unwrap();
    let core = cortex_m::Peripherals::take().unwrap();
    let mut rcc = device.RCC.constrain();
    let mut flash = device.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(64.mhz())
        .pclk1(32.mhz())
        .pclk2(36.mhz())
        .freeze(&mut flash.acr);
    let gpioa = device.GPIOA.split(&mut rcc.ahb);
    let gpiob = device.GPIOB.split(&mut rcc.ahb);
    let mut serial = device
        .USART1
        .serial((gpioa.pa9, gpioa.pa10), Bps(115200), clocks);
    serial.listen(serial::Event::Rxne);
    let (mut tx, _rx) = serial.split();
    // COBS frame
    tx.write(0x00).unwrap();
    let mut l = Logger { tx };
    write!(l, "logger ok\r\n");
    let mut delay = delay::Delay::new(core.SYST, clocks);
    // SPI1
    let ncs = gpiob.pb9.output().push_pull();
    let scl_sck = gpiob.pb3;
    let sda_sdi_mosi = gpiob.pb5;
    let ad0_sdo_miso = gpiob.pb4;
    let spi = device.SPI1.spi(
        (scl_sck, ad0_sdo_miso, sda_sdi_mosi),
        mpu9250::MODE,
        1.mhz(),
        clocks,
    );
    let mut mpu = Mpu9250::imu_default(spi, ncs, &mut delay).unwrap();

    write!(l, "starting loop...\r\n");

    let mut flag = true;
    loop {
        if flag {
            match mpu.temp() {
                Ok(t) => {
                    write!(l, "Temp: {:?}\r\n", t);
                }
                Err(e) => {
                    write!(l, "Error: {:?}\r\n", e);
                }
            }
        } else {
            match mpu.raw_temp() {
                Ok(t) => {
                    write!(l, "Raw Temp: {:?}\r\n", t);
                }
                Err(e) => {
                    write!(l, "Error: {:?}\r\n", e);
                }
            }
        }
        flag = !flag;
        for _ in 0..10 {
            delay.delay_ms(250u32);
        }
    }
}

struct Logger<W: ehal::serial::Write<u8>> {
    tx: W,
}
impl<W: ehal::serial::Write<u8>> fmt::Write for Logger<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            match self.write_char(c) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        match self.tx.flush() {
            Ok(_) => {}
            Err(_) => {}
        };

        Ok(())
    }

    fn write_char(&mut self, s: char) -> fmt::Result {
        match nb::block!(self.tx.write(s as u8)) {
            Ok(_) => {}
            Err(_) => {}
        }
        Ok(())
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
