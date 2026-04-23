use config::ConfigEvent;
use defmt::info;
use embassy_time::{Duration, Ticker};
use static_cell::StaticCell;

use crate::{
    config_task::CONFIG_EVENT_CHANNEL,
    hardware::abstractions::{Button, QeiExt},
};

const DEBOUNCE_TICKS: u8 = 5;

pub struct ButtonState<'a> {
    button: &'a dyn Button,
    last_raw: bool,
    stable: bool,
    counter: u8,
}

impl<'a> ButtonState<'a> {
    pub fn new(button: &'a dyn Button) -> Self {
        Self {
            button,
            last_raw: false,
            stable: false,
            counter: 0,
        }
    }

    pub fn process(&mut self) -> bool {
        let mut just_pressed = false;

        let raw = self.button.is_pressed();

        if raw == self.last_raw {
            if self.counter < DEBOUNCE_TICKS {
                // Count how many ticks the button has been stable
                self.counter += 1;
            } else if self.stable != raw {
                // If it's been stable long enough and it's changed, we update
                self.stable = raw;

                if raw {
                    just_pressed = true;
                }
            }
        } else {
            self.counter = 0;
        }

        self.last_raw = raw;

        just_pressed
    }
}

pub struct EncoderState<'a> {
    qei: &'a dyn QeiExt,
    last: u16,
}

impl<'a> EncoderState<'a> {
    pub fn new(qei: &'a dyn QeiExt) -> Self {
        Self { qei, last: 0 }
    }

    pub fn process(&mut self) -> Option<i8> {
        let current = self.qei.count();

        let diff = (current.wrapping_sub(self.last) as i16).clamp(-128, 127) as i8;

        self.last = current;

        if diff == 0 { None } else { Some(diff) }
    }
}

pub struct InputTaskState<'a> {
    button_next_page: ButtonState<'a>,
    button_prev_page: ButtonState<'a>,
    encoder0: EncoderState<'a>,
    encoder1: EncoderState<'a>,
    encoder2: EncoderState<'a>,
}

impl<'a> InputTaskState<'a> {
    pub fn new(
        button_next_page: &'a dyn Button,
        button_prev_page: &'a dyn Button,
        encoder0: &'a dyn QeiExt,
        encoder1: &'a dyn QeiExt,
        encoder2: &'a dyn QeiExt,
    ) -> Self {
        Self {
            button_next_page: ButtonState::new(button_next_page),
            button_prev_page: ButtonState::new(button_prev_page),
            encoder0: EncoderState::new(encoder0),
            encoder1: EncoderState::new(encoder1),
            encoder2: EncoderState::new(encoder2),
        }
    }
}

static INPUT_STATE: StaticCell<InputTaskState> = StaticCell::new();

pub fn spawn_config_hardware_tasks(
    spawner: &embassy_executor::Spawner,
    input_hardware: crate::hardware::InputHardware,
) {
    spawner
        .spawn(input_task(INPUT_STATE.init(InputTaskState::new(
            input_hardware.button_next_page,
            input_hardware.button_prev_page,
            input_hardware.encoder0,
            input_hardware.encoder1,
            input_hardware.encoder2,
        ))))
        .unwrap();
}

#[embassy_executor::task]
pub async fn input_task(state: &'static mut InputTaskState<'static>) {
    info!("Input task started");

    let sender = CONFIG_EVENT_CHANNEL.sender();

    let mut ticker = Ticker::every(Duration::from_millis(5));

    loop {
        ticker.next().await;

        if state.button_next_page.process() {
            sender.try_send(ConfigEvent::PageChange { amount: 1 }).ok();
        }

        if state.button_prev_page.process() {
            sender.try_send(ConfigEvent::PageChange { amount: -1 }).ok();
        }

        if let Some(diff) = state.encoder0.process() {
            sender
                .try_send(ConfigEvent::EncoderChange {
                    encoder: 0,
                    amount: diff,
                })
                .ok();
        }

        if let Some(diff) = state.encoder1.process() {
            sender
                .try_send(ConfigEvent::EncoderChange {
                    encoder: 1,
                    amount: diff,
                })
                .ok();
        }

        if let Some(diff) = state.encoder2.process() {
            sender
                .try_send(ConfigEvent::EncoderChange {
                    encoder: 2,
                    amount: diff,
                })
                .ok();
        }
    }
}
