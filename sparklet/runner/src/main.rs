#![no_std]
#![no_main]

mod hardware;
mod midi_task;

use defmt::info;
use embassy_executor::Executor;
use static_cell::StaticCell;

use defmt_rtt as _;
use embassy_stm32 as _;
use panic_probe as _;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let hardware = hardware::Hardware::get();

    let executor = EXECUTOR.init(embassy_executor::Executor::new());

    midi_task::setup_task(executor, hardware.midi_uart_buffered);

    info!("Hello, world!");

    panic!();
}
