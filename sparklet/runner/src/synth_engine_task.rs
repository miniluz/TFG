use defmt::info;
use embassy_executor::SpawnToken;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use synth_engine::{Q15, SynthEngine};

use crate::midi_task::{MIDI_CHANNEL_SIZE, MIDI_TASK_CHANNEL};

const VOICE_BANK_SIZE: usize = 4;
const RUN_RATE_HZ: u16 = 1000;

pub struct SynthEngineTaskState<'a> {
    synth_engine: SynthEngine<'a, CriticalSectionRawMutex, MIDI_CHANNEL_SIZE, VOICE_BANK_SIZE>,
}

impl<'a> SynthEngineTaskState<'a> {
    pub fn new(
        synth_engine: SynthEngine<'a, CriticalSectionRawMutex, MIDI_CHANNEL_SIZE, VOICE_BANK_SIZE>,
    ) -> SynthEngineTaskState<'a> {
        SynthEngineTaskState { synth_engine }
    }
}

pub static SYNTH_ENGINE_TASK_STATE: StaticCell<SynthEngineTaskState> = StaticCell::new();

pub fn create_task() -> SpawnToken<impl Sized> {
    let synth_engine = SynthEngine::new(MIDI_TASK_CHANNEL.receiver());

    synth_engine_task(SYNTH_ENGINE_TASK_STATE.init(SynthEngineTaskState::new(synth_engine)))
}

#[embassy_executor::task]
pub async fn synth_engine_task(state: &'static mut SynthEngineTaskState<'static>) {
    let mut buffer = [Q15::ZERO; 16];
    let mut counter: u16 = 0;

    loop {
        state.synth_engine.render_samples(&mut buffer);

        if counter == 0 {
            info!("Voice bank state");
            info!("{:?}", state.synth_engine.get_voice_bank());
        }

        counter = (counter + 1) % RUN_RATE_HZ;

        Timer::after(Duration::from_hz(RUN_RATE_HZ.into())).await;
    }
}
