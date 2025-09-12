#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32h7xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let core_peripherals = cortex_m::Peripherals::take().unwrap();
    let device_peripherals = pac::Peripherals::take().unwrap();

    rprintln!("Setup PWR...                  ");
    let pwr = device_peripherals.PWR.constrain();
    let pwrcfg = pwr.freeze();

    rprintln!("Setup RCC...                  ");
    let rcc = device_peripherals.RCC.constrain();
    let ccdr = rcc
        .sys_ck(100.MHz())
        .freeze(pwrcfg, &device_peripherals.SYSCFG);

    rprintln!("");
    rprintln!("stm32h7xx-hal example - Blinky");
    rprintln!("");

    let gpioe = device_peripherals.GPIOE.split(ccdr.peripheral.GPIOE);

    let mut led = gpioe.pe1.into_push_pull_output();

    let mut delay = core_peripherals.SYST.delay(ccdr.clocks);

    loop {
        led.set_high();
        delay.delay_ms(500_u16);

        led.set_low();
        delay.delay_ms(500_u16);
    }
}
