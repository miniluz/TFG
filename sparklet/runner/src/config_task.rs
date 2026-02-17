use config::{ConfigEvent, ConfigManager};
use defmt::info;
use embassy_executor::SpawnToken;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Timer};
use static_cell::StaticCell;

use crate::synth_engine_task::CONFIG_SIGNAL;

const CONFIG_PAGE_COUNT: usize = 2;
const CONFIG_ENCODER_COUNT: usize = 3;
const UPDATE_RATE_HZ: u32 = 100;

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

    let mut attack = 40u8;
    let mut attack_dir = 1i8;

    let mut sustain = 200u8;
    let mut sustain_dir = 1i8;

    let mut release = 127u8;
    let mut release_dir = 1i8;

    const ATTACK_UPDATE_EVERY: u32 = 1;
    const SUSTAIN_UPDATE_EVERY: u32 = 2;
    const RELEASE_UPDATE_EVERY: u32 = 3;

    let osc_change_interval_ms = 10000u64;
    let mut last_osc_type = 1u8;

    let mut counter = 0u32;

    loop {
        let elapsed_ms = start_time.elapsed().as_millis();

        if counter.is_multiple_of(ATTACK_UPDATE_EVERY) {
            sender
                .send(ConfigEvent::EncoderChange {
                    encoder: 0,
                    amount: attack_dir,
                })
                .await;
            attack = attack.saturating_add_signed(attack_dir);
            if attack == 255 || attack == 0 {
                attack_dir = -attack_dir;
            }
        }

        if counter.is_multiple_of(SUSTAIN_UPDATE_EVERY) {
            sender
                .send(ConfigEvent::EncoderChange {
                    encoder: 1,
                    amount: sustain_dir,
                })
                .await;
            sustain = sustain.saturating_add_signed(sustain_dir);
            if sustain == 255 || sustain == 0 {
                sustain_dir = -sustain_dir;
            }
        }

        if counter.is_multiple_of(RELEASE_UPDATE_EVERY) {
            sender
                .send(ConfigEvent::EncoderChange {
                    encoder: 2,
                    amount: release_dir,
                })
                .await;
            release = release.saturating_add_signed(release_dir);
            if release == 255 || release == 0 {
                release_dir = -release_dir;
            }
        }

        let target_osc_type = ((elapsed_ms / osc_change_interval_ms) as u8) % 4;
        if target_osc_type != last_osc_type {
            sender.send(ConfigEvent::PageChange { amount: 1 }).await;

            let delta = (target_osc_type as i16 - last_osc_type as i16).signum() as i8;
            sender
                .send(ConfigEvent::EncoderChange {
                    encoder: 0,
                    amount: delta,
                })
                .await;
            last_osc_type = target_osc_type;

            sender.send(ConfigEvent::PageChange { amount: -1 }).await;

            let osc_name = match target_osc_type {
                0 => "Sine",
                1 => "Sawtooth",
                2 => "Square",
                _ => "Triangle",
            };
            info!(
                "Encoder Simulator: Switching to {} waveform at {}ms",
                osc_name, elapsed_ms
            );
        }

        if counter.is_multiple_of(UPDATE_RATE_HZ * 5) {
            info!(
                "Encoder Simulator: t={}ms Attack={}, Sustain={}, Release={}, Osc={}",
                elapsed_ms, attack, sustain, release, last_osc_type
            );
        }

        counter = counter.wrapping_add(1);

        Timer::after(Duration::from_hz(UPDATE_RATE_HZ.into())).await;
    }
}
