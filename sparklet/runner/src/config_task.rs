use config::{ConfigEvent, ConfigManager};
use defmt::info;
use embassy_executor::SpawnToken;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use static_cell::StaticCell;

use crate::synth_engine_task::CONFIG_SIGNAL;

const BASE_PAGE_COUNT: usize = 2;

#[cfg(feature = "octave-filter")]
const OCTAVE_FILTER_PAGE_COUNT: usize = 2;
#[cfg(not(feature = "octave-filter"))]
const OCTAVE_FILTER_PAGE_COUNT: usize = 0;

const CONFIG_PAGE_COUNT: usize = BASE_PAGE_COUNT + OCTAVE_FILTER_PAGE_COUNT;
const CONFIG_ENCODER_COUNT: usize = 3;

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

#[embassy_executor::task]
pub async fn config_manager_task(state: &'static mut ConfigManagerTaskState<'static>) {
    info!("Config Manager: Task starting");

    let receiver = CONFIG_EVENT_CHANNEL.receiver();

    loop {
        let event = receiver.receive().await;
        state.config_manager.handle_event(event);
    }
}
