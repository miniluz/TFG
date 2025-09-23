use defmt::info;
use embassy_time::Timer;

use embassy_stm32::gpio::Output;
use static_cell::StaticCell;

pub static NUM_LEDS: usize = 3;

pub struct State<'a> {
    leds: [Output<'a>; NUM_LEDS],
}

impl State<'_> {
    pub fn new<'a>(leds: [Output<'a>; NUM_LEDS]) -> State<'a> {
        State { leds }
    }
}

pub static STATE: StaticCell<State> = StaticCell::new();

#[embassy_executor::task]
pub async fn blinky_task(state: &'static mut State<'static>) {
    loop {
        info!("Blinky!");
        state.leds[0].toggle();
        Timer::after_millis(1000).await
    }
}
