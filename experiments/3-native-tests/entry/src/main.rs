#![no_std]
#![no_main]

use adder_cmsis::CmsisAddOperations;
use adder_interface::AddOperations;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hey!");
    info!("Added 1+1: {}", CmsisAddOperations::add(1, 1));

    panic!();
}
