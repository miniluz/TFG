use defmt::info;
use embassy_time::Timer;

use embassy_stm32::gpio::Output;
use static_cell::StaticCell;

pub static NUM_LEDS: usize = 3;

pub struct BlinkyState<'a> {
    leds: [Output<'a>; NUM_LEDS],
}

impl BlinkyState<'_> {
    pub fn new<'a>(leds: [Output<'a>; NUM_LEDS]) -> BlinkyState<'a> {
        BlinkyState { leds }
    }
}

pub static BLINKY_STATE: StaticCell<BlinkyState> = StaticCell::new();

#[embassy_executor::task]
pub async fn blinky_task(state: &'static mut BlinkyState<'static>) {
    loop {
        info!("Blinky!");
        state.leds[0].toggle();
        Timer::after_millis(1000).await
    }
}
