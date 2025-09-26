use defmt::info;
use embassy_stm32::exti::ExtiInput;
use embassy_time::Timer;
use static_cell::StaticCell;

pub struct ButtonState<'a> {
    button: ExtiInput<'a>,
}

impl ButtonState<'_> {
    pub fn new<'a>(button: ExtiInput<'a>) -> ButtonState<'a> {
        ButtonState { button }
    }
}

pub static BUTTON_STATE: StaticCell<ButtonState> = StaticCell::new();

#[embassy_executor::task]
pub async fn button_task(state: &'static mut ButtonState<'static>) {
    loop {
        state.button.wait_for_high().await;
        info!("Button pressed!");
        Timer::after_millis(200).await;
        state.button.wait_for_low().await;
        info!("Button unpressed!");
    }
}
