#![no_std]
#![no_main]

mod blinky;
mod hardware;

use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Executor;
use static_cell::StaticCell;

use crate::{
    blinky::{STATE, State, blinky_task},
    hardware::Hardware,
};

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let hardware = Hardware::get();

    let executor = EXECUTOR.init(embassy_executor::Executor::new());
    executor.run(|spawner| {
        spawner
            .spawn(blinky_task(STATE.init(State::new([
                hardware.left_led,
                hardware.middle_led,
                hardware.right_led,
            ]))))
            .ok();
    })
}
