use config::ConfigEvent;
use defmt::info;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Input;
use embassy_time::Timer;
use static_cell::StaticCell;

use crate::config_task::CONFIG_EVENT_CHANNEL;

#[allow(dead_code)]
pub enum Polarity {
    ActiveLow,
    ActiveHigh,
}

pub struct Button<'a> {
    button: ExtiInput<'a>,
    polarity: Polarity,
}

impl<'a> Button<'a> {
    pub fn new(button: ExtiInput<'a>, polarity: Polarity) -> Button<'a> {
        Button { button, polarity }
    }

    pub async fn wait_for_pressed(&mut self) {
        match self.polarity {
            Polarity::ActiveHigh => self.button.wait_for_high().await,
            Polarity::ActiveLow => self.button.wait_for_low().await,
        }
    }

    pub async fn wait_for_released(&mut self) {
        match self.polarity {
            Polarity::ActiveHigh => self.button.wait_for_low().await,
            Polarity::ActiveLow => self.button.wait_for_high().await,
        }
    }
}

pub struct ButtonTaskState<'a> {
    button: Button<'a>,
    page_delta: i8,
}

impl<'a> ButtonTaskState<'a> {
    pub fn new(button: Button<'a>, page_delta: i8) -> Self {
        Self {
            button,
            page_delta,
        }
    }
}

pub struct EncoderTaskState<'a> {
    encoder_exti: ExtiInput<'a>,
    encoder_input: Input<'a>,
    encoder_index: u8,
}

impl<'a> EncoderTaskState<'a> {
    pub fn new(encoder_exti: ExtiInput<'a>, encoder_input: Input<'a>, encoder_index: u8) -> Self {
        Self {
            encoder_exti,
            encoder_input,
            encoder_index,
        }
    }
}

static BUTTON_PAGE_DOWN_STATE: StaticCell<ButtonTaskState> = StaticCell::new();
static BUTTON_PAGE_UP_STATE: StaticCell<ButtonTaskState> = StaticCell::new();
static ENCODER0_STATE: StaticCell<EncoderTaskState> = StaticCell::new();
static ENCODER1_STATE: StaticCell<EncoderTaskState> = StaticCell::new();
static ENCODER2_STATE: StaticCell<EncoderTaskState> = StaticCell::new();

pub fn spawn_config_hardware_tasks(
    spawner: &embassy_executor::Spawner,
    input_hardware: crate::hardware::InputHardware<'static>,
) {
    let button_page_down = Button::new(input_hardware.button_page_down, Polarity::ActiveLow);
    let button_page_up = Button::new(input_hardware.button_page_up, Polarity::ActiveLow);

    spawner
        .spawn(button_page_down_task(
            BUTTON_PAGE_DOWN_STATE.init(ButtonTaskState::new(button_page_down, -1)),
        ))
        .unwrap();

    spawner
        .spawn(button_page_up_task(
            BUTTON_PAGE_UP_STATE.init(ButtonTaskState::new(button_page_up, 1)),
        ))
        .unwrap();

    spawner
        .spawn(encoder0_task(ENCODER0_STATE.init(EncoderTaskState::new(
            input_hardware.encoder0_exti,
            input_hardware.encoder0_input,
            0,
        ))))
        .unwrap();

    spawner
        .spawn(encoder1_task(ENCODER1_STATE.init(EncoderTaskState::new(
            input_hardware.encoder1_exti,
            input_hardware.encoder1_input,
            1,
        ))))
        .unwrap();

    spawner
        .spawn(encoder2_task(ENCODER2_STATE.init(EncoderTaskState::new(
            input_hardware.encoder2_exti,
            input_hardware.encoder2_input,
            2,
        ))))
        .unwrap();
}

#[embassy_executor::task]
pub async fn button_page_down_task(state: &'static mut ButtonTaskState<'static>) {
    info!("Button task started (page delta: {})", state.page_delta);

    let sender = CONFIG_EVENT_CHANNEL.sender();

    loop {
        state.button.wait_for_pressed().await;
        info!("Button pressed! Sending page change: {}", state.page_delta);

        sender
            .send(ConfigEvent::PageChange {
                amount: state.page_delta,
            })
            .await;

        Timer::after_millis(200).await;
        state.button.wait_for_released().await;
    }
}

#[embassy_executor::task]
pub async fn button_page_up_task(state: &'static mut ButtonTaskState<'static>) {
    info!("Button task started (page delta: {})", state.page_delta);

    let sender = CONFIG_EVENT_CHANNEL.sender();

    loop {
        state.button.wait_for_pressed().await;
        info!("Button pressed! Sending page change: {}", state.page_delta);

        sender
            .send(ConfigEvent::PageChange {
                amount: state.page_delta,
            })
            .await;

        Timer::after_millis(200).await;
        state.button.wait_for_released().await;
    }
}

#[embassy_executor::task]
pub async fn encoder0_task(state: &'static mut EncoderTaskState<'static>) {
    info!("Encoder {} task started", state.encoder_index);

    let sender = CONFIG_EVENT_CHANNEL.sender();

    loop {
        state.encoder_exti.wait_for_any_edge().await;

        let a_state = state.encoder_exti.is_high();
        let b_state = state.encoder_input.is_high();

        let amount = if a_state != b_state { 1 } else { -1 };

        sender
            .send(ConfigEvent::EncoderChange {
                encoder: state.encoder_index,
                amount,
            })
            .await;
    }
}

#[embassy_executor::task]
pub async fn encoder1_task(state: &'static mut EncoderTaskState<'static>) {
    info!("Encoder {} task started", state.encoder_index);

    let sender = CONFIG_EVENT_CHANNEL.sender();

    loop {
        state.encoder_exti.wait_for_any_edge().await;

        let a_state = state.encoder_exti.is_high();
        let b_state = state.encoder_input.is_high();

        let amount = if a_state != b_state { 1 } else { -1 };

        sender
            .send(ConfigEvent::EncoderChange {
                encoder: state.encoder_index,
                amount,
            })
            .await;
    }
}

#[embassy_executor::task]
pub async fn encoder2_task(state: &'static mut EncoderTaskState<'static>) {
    info!("Encoder {} task started", state.encoder_index);

    let sender = CONFIG_EVENT_CHANNEL.sender();

    loop {
        state.encoder_exti.wait_for_any_edge().await;

        let a_state = state.encoder_exti.is_high();
        let b_state = state.encoder_input.is_high();

        let amount = if a_state != b_state { 1 } else { -1 };

        sender
            .send(ConfigEvent::EncoderChange {
                encoder: state.encoder_index,
                amount,
            })
            .await;
    }
}
