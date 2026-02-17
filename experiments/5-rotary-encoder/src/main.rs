#![no_std]
#![no_main]

mod encoder_task;
mod hardware;

use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Executor;
use static_cell::StaticCell;

use crate::{
    encoder_task::{encoder_task, position_logger_task, EncoderTaskState, ENCODER_STATE},
    hardware::Hardware,
};

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Setting up hardware");
    let hardware = Hardware::get();

    info!("Setting up executor");
    let executor = EXECUTOR.init(embassy_executor::Executor::new());

    executor.run(|spawner| {
        info!("Spawning encoder task");
        spawner
            .spawn(encoder_task(ENCODER_STATE.init(EncoderTaskState::new(
                hardware.encoder_a,
                hardware.encoder_b,
            ))))
            .unwrap();

        info!("Spawning position logger task");
        spawner.spawn(position_logger_task()).unwrap();
    })
}
