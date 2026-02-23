use config::{ConfigEvent, ConfigManager};
use defmt::info;
use embassy_executor::SpawnToken;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Timer};
use static_cell::StaticCell;

use crate::synth_engine_task::CONFIG_SIGNAL;

const BASE_PAGE_COUNT: usize = 2;

#[cfg(feature = "octave-filter")]
const OCTAVE_FILTER_PAGE_COUNT: usize = 2;
#[cfg(not(feature = "octave-filter"))]
const OCTAVE_FILTER_PAGE_COUNT: usize = 0;

const CONFIG_PAGE_COUNT: usize = BASE_PAGE_COUNT + OCTAVE_FILTER_PAGE_COUNT;
const CONFIG_ENCODER_COUNT: usize = 3;
const UPDATE_RATE_HZ: u32 = 500;

const CONFIG_CHANNEL_SIZE: usize = 32;
pub static CONFIG_EVENT_CHANNEL: Channel<
    CriticalSectionRawMutex,
    ConfigEvent,
    CONFIG_CHANNEL_SIZE,
> = Channel::new();

pub struct ConfigManagerTaskState<'cfg> {
    config_manager:
        ConfigManager<'cfg, CriticalSectionRawMutex, CONFIG_PAGE_COUNT, CONFIG_ENCODER_COUNT>,
}

impl<'cfg> ConfigManagerTaskState<'cfg> {
    pub fn new(
        config_manager: ConfigManager<
            'cfg,
            CriticalSectionRawMutex,
            CONFIG_PAGE_COUNT,
            CONFIG_ENCODER_COUNT,
        >,
    ) -> Self {
        Self { config_manager }
    }
}

pub static CONFIG_MANAGER_TASK_STATE: StaticCell<ConfigManagerTaskState> = StaticCell::new();

pub fn create_config_manager_task() -> SpawnToken<impl Sized> {
    let config_manager = ConfigManager::new(&CONFIG_SIGNAL);

    config_manager_task(CONFIG_MANAGER_TASK_STATE.init(ConfigManagerTaskState::new(config_manager)))
}

pub fn create_encoder_simulator_task() -> SpawnToken<impl Sized> {
    encoder_simulator_task()
}

#[embassy_executor::task]
pub async fn config_manager_task(state: &'static mut ConfigManagerTaskState<'static>) {
    info!("Config Manager: Task starting");

    let receiver = CONFIG_EVENT_CHANNEL.receiver();

    loop {
        let event = receiver.receive().await;
        state.config_manager.handle_event(event);
    }
}

#[embassy_executor::task]
pub async fn encoder_simulator_task() {
    info!("Encoder Simulator: Task starting at {} Hz", UPDATE_RATE_HZ);

    let start_time = Instant::now();
    let sender = CONFIG_EVENT_CHANNEL.sender();

    #[cfg(feature = "octave-filter")]
    let mut encoder_states = [
        [(0u8, 1i8, 5u32), (0u8, 1i8, 6u32), (0u8, 1i8, 7u32)],
        [(1u8, 0i8, 0u32), (0u8, 1i8, 8u32), (0u8, 1i8, 9u32)],
        [(0u8, 1i8, 10u32), (0u8, 1i8, 11u32), (0u8, 1i8, 12u32)],
        [(0u8, 1i8, 13u32), (0u8, 1i8, 14u32), (0u8, 1i8, 15u32)],
    ];

    #[cfg(not(feature = "octave-filter"))]
    let mut encoder_states = [
        [(0u8, 1i8, 5u32), (0u8, 1i8, 6u32), (0u8, 1i8, 7u32)],
        [(1u8, 0i8, 0u32), (0u8, 1i8, 8u32), (0u8, 1i8, 9u32)],
    ];

    let mut counter = 0u32;
    let mut current_page = 0u8;

    loop {
        let elapsed_ms = start_time.elapsed().as_millis();

        for (page, page_states) in encoder_states.iter_mut().enumerate() {
            for (encoder, (value, dir, update_every)) in page_states.iter_mut().enumerate() {

                if page == 1 && encoder == 0 {
                    continue;
                }

                if counter.is_multiple_of(*update_every) {
                    let target_page = page as u8;
                    if target_page != current_page {
                        let page_delta = (target_page as i8) - (current_page as i8);
                        sender
                            .send(ConfigEvent::PageChange { amount: page_delta })
                            .await;
                        current_page = target_page;
                    }

                    sender
                        .send(ConfigEvent::EncoderChange {
                            encoder: encoder as u8,
                            amount: *dir,
                        })
                        .await;

                    *value = value.saturating_add_signed(*dir);
                    if *value == 255 || *value == 0 {
                        *dir = -*dir;
                    }
                }
            }
        }

        // Log status every 5 seconds
        if counter.is_multiple_of(UPDATE_RATE_HZ * 5) {
            #[cfg(feature = "octave-filter")]
            info!(
                "Encoder Simulator: t={}ms P0=[{},{},{}] P1=[{},{},{}] P2=[{},{},{}] P3=[{},{},{}]",
                elapsed_ms,
                encoder_states[0][0].0,
                encoder_states[0][1].0,
                encoder_states[0][2].0,
                encoder_states[1][0].0,
                encoder_states[1][1].0,
                encoder_states[1][2].0,
                encoder_states[2][0].0,
                encoder_states[2][1].0,
                encoder_states[2][2].0,
                encoder_states[3][0].0,
                encoder_states[3][1].0,
                encoder_states[3][2].0,
            );

            #[cfg(not(feature = "octave-filter"))]
            info!(
                "Encoder Simulator: t={}ms P0=[{},{},{}] P1=[{},{},{}]",
                elapsed_ms,
                encoder_states[0][0].0,
                encoder_states[0][1].0,
                encoder_states[0][2].0,
                encoder_states[1][0].0,
                encoder_states[1][1].0,
                encoder_states[1][2].0,
            );
        }

        counter = counter.wrapping_add(1);

        Timer::after(Duration::from_hz(UPDATE_RATE_HZ.into())).await;
    }
}
