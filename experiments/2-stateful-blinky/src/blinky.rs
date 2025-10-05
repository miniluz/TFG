use defmt::info;
use embassy_time::{Duration, WithTimeout};

use embassy_stm32::gpio::Output;
use static_cell::StaticCell;

use crate::button::SIGNAL;

pub static NUM_LEDS: usize = 3;

#[derive(Copy, Clone)]
enum Led {
    Left,
    Middle,
    Right,
}

impl Led {
    fn next(self) -> Led {
        match self {
            Led::Left => Led::Middle,
            Led::Middle => Led::Right,
            Led::Right => Led::Left,
        }
    }
}

impl From<Led> for usize {
    fn from(led: Led) -> Self {
        match led {
            Led::Left => 0,
            Led::Middle => 1,
            Led::Right => 2,
        }
    }
}

pub struct BlinkyState<'a> {
    leds: [Output<'a>; NUM_LEDS],
    current_led: Led,
}

impl BlinkyState<'_> {
    pub fn new<'a>(leds: [Output<'a>; NUM_LEDS]) -> BlinkyState<'a> {
        BlinkyState {
            leds,
            current_led: Led::Left,
        }
    }
}

pub static BLINKY_STATE: StaticCell<BlinkyState> = StaticCell::new();

#[embassy_executor::task]
pub async fn blinky_task(state: &'static mut BlinkyState<'static>) {
    loop {
        info!("Blinky!");

        let current_led: usize = state.current_led.into();
        state.leds[current_led].toggle();

        let result = SIGNAL
            .wait()
            .with_timeout(Duration::from_millis(500))
            .await
            .ok();

        if let Some(()) = result {
            state.leds[current_led].set_low();
            state.current_led = state.current_led.next();
        }
    }
}
