#![no_std]
#![no_main]

mod blinky;
mod button;
mod hardware;

use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Executor;
use static_cell::StaticCell;

use crate::{
    blinky::{BLINKY_STATE, BlinkyState, blinky_task},
    button::{BUTTON_STATE, ButtonState, button_task},
    hardware::Hardware,
};

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let hardware = Hardware::get();

    let executor = EXECUTOR.init(embassy_executor::Executor::new());
    executor.run(|spawner| {
        spawner
            .spawn(blinky_task(BLINKY_STATE.init(BlinkyState::new([
                hardware.left_led,
                hardware.middle_led,
                hardware.right_led,
            ]))))
            .ok();
        spawner
            .spawn(button_task(
                BUTTON_STATE.init(ButtonState::new(hardware.button)),
            ))
            .ok();
    })
}
