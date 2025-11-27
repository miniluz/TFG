#![no_std]
#![no_main]

mod hardware;
mod midi_task;
mod synth_engine_task;

use defmt::info;
use embassy_executor::Executor;
use static_cell::StaticCell;

use defmt_rtt as _;
use embassy_stm32 as _;
use panic_probe as _;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Setting up hardware");
    let hardware = hardware::Hardware::get();

    info!("Setting up executor");
    let executor = EXECUTOR.init(embassy_executor::Executor::new());

    info!("Creating MIDI task");
    let midi_task = midi_task::create_midi_task(hardware.midi_hardware);

    info!("Creating synth engine task");
    let synth_engine_task = synth_engine_task::create_task();

    info!("Setting up tasks in executors...");
    executor.run(|spawner| {
        spawner.spawn(midi_task).unwrap();
        spawner.spawn(synth_engine_task).unwrap();
    });
}
