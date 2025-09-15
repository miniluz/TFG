#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use defmt::info;
use embassy_executor::Executor;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use static_cell::StaticCell;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

struct State<'a> {
    led: Output<'a>,
}

static STATE: StaticCell<State> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Initializing");
    let peripherals = embassy_stm32::init(Default::default());

    let led = Output::new(peripherals.PB0, Level::Low, Speed::Low);

    let executor = EXECUTOR.init(embassy_executor::Executor::new());
    executor.run(|spawner| {
        spawner.spawn(blinky(STATE.init(State { led }))).ok();
    })
}

#[embassy_executor::task]
async fn blinky(state: &'static mut State<'static>) {
    loop {
        info!("Blinky!");
        state.led.toggle();
        Timer::after_millis(1000).await
    }
}
