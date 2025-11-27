use cmsis_native::CmsisNativeOperations;
use defmt::info;
use embassy_executor::SpawnToken;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use synth_engine::{wavetable::saw_wavetable::SAW_WAVETABLE, Q15, SynthEngine, WINDOW_SIZE};

use crate::midi_task::{MIDI_CHANNEL_SIZE, MIDI_TASK_CHANNEL};

const VOICE_BANK_SIZE: usize = 4;
const RUN_RATE_HZ: u16 = 1000;

const ATTACK_CONFIG: u8 = 40;
const DECAY_RELEASE_CONFIG: u8 = 127;
const SUSTAIN_CONFIG: u8 = 200;

pub struct SynthEngineTaskState<'ch, 'wt> {
    synth_engine: SynthEngine<'ch, 'wt, CriticalSectionRawMutex, MIDI_CHANNEL_SIZE, VOICE_BANK_SIZE>,
}

impl<'ch, 'wt> SynthEngineTaskState<'ch, 'wt> {
    pub fn new(
        synth_engine: SynthEngine<'ch, 'wt, CriticalSectionRawMutex, MIDI_CHANNEL_SIZE, VOICE_BANK_SIZE>,
    ) -> SynthEngineTaskState<'ch, 'wt> {
        SynthEngineTaskState { synth_engine }
    }
}

pub static SYNTH_ENGINE_TASK_STATE: StaticCell<SynthEngineTaskState> = StaticCell::new();

pub fn create_task() -> SpawnToken<impl Sized> {
    let synth_engine = SynthEngine::new(
        MIDI_TASK_CHANNEL.receiver(),
        &SAW_WAVETABLE,
        SUSTAIN_CONFIG,
        ATTACK_CONFIG,
        DECAY_RELEASE_CONFIG,
    );

    synth_engine_task(SYNTH_ENGINE_TASK_STATE.init(SynthEngineTaskState::new(synth_engine)))
}

#[embassy_executor::task]
pub async fn synth_engine_task(state: &'static mut SynthEngineTaskState<'static, 'static>) {
    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    let mut counter: u16 = 0;

    loop {
        state.synth_engine.render_samples::<CmsisNativeOperations>(&mut buffer);

        if counter == 0 {
            info!("Voice bank state");
        }

        counter = (counter + 1) % RUN_RATE_HZ;

        Timer::after(Duration::from_hz(RUN_RATE_HZ.into())).await;
    }
}
