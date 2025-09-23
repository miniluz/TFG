#![no_std]
#![no_main]

use adder::add;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hey!");
    info!("Added 1+1: {}", add(1, 1));

    panic!();
}
