use defmt::info;
use embassy_stm32::exti::ExtiInput;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use static_cell::StaticCell;

pub struct EncoderTaskState<'a> {
    encoder_a: ExtiInput<'a>,
    encoder_b: ExtiInput<'a>,
    position: u8,
}

impl<'a> EncoderTaskState<'a> {
    pub fn new(encoder_a: ExtiInput<'a>, encoder_b: ExtiInput<'a>) -> Self {
        Self {
            encoder_a,
            encoder_b,
            position: 0,
        }
    }
}

pub static ENCODER_STATE: StaticCell<EncoderTaskState> = StaticCell::new();

const CHANNEL_SIZE: usize = 16;
pub static POSITION_CHANNEL: Channel<CriticalSectionRawMutex, u8, CHANNEL_SIZE> = Channel::new();

#[embassy_executor::task]
pub async fn encoder_task(state: &'static mut EncoderTaskState<'static>) {
    info!("Encoder task started");
    info!("Initial A: {}, B: {}", state.encoder_a.is_high(), state.encoder_b.is_high());

    let sender = POSITION_CHANNEL.sender();

    loop {
        state.encoder_a.wait_for_any_edge().await;

        info!("Edge detected! A: {}, B: {}", state.encoder_a.is_high(), state.encoder_b.is_high());

        let a_state = state.encoder_a.is_high();
        let b_state = state.encoder_b.is_high();

        if a_state != b_state {
            state.position = state.position.wrapping_add(1);
        } else {
            state.position = state.position.wrapping_sub(1);
        }

        info!("Position: {}", state.position);
        sender.try_send(state.position).ok();
    }
}

#[embassy_executor::task]
pub async fn position_logger_task() {
    info!("Position logger task started");

    let receiver = POSITION_CHANNEL.receiver();

    loop {
        let position = receiver.receive().await;
        info!("Position update received: {}", position);
    }
}
