#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_stm32 as _;
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello, world!");

    panic!();
}
