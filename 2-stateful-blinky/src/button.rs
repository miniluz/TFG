use defmt::info;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::Timer;
use static_cell::StaticCell;

use crate::hardware::Button;

pub struct ButtonState<'a> {
    button: Button<'a>,
}

impl ButtonState<'_> {
    pub fn new<'a>(button: Button<'a>) -> ButtonState<'a> {
        ButtonState { button }
    }
}

pub static BUTTON_STATE: StaticCell<ButtonState> = StaticCell::new();

pub static SIGNAL: Signal<ThreadModeRawMutex, ()> = Signal::new();

#[embassy_executor::task]
pub async fn button_task(state: &'static mut ButtonState<'static>) {
    loop {
        state.button.wait_for_pressed().await;
        info!("Button pressed!");
        SIGNAL.signal(());
        Timer::after_millis(200).await;
        state.button.wait_for_released().await;
        info!("Button unpressed!");
    }
}
