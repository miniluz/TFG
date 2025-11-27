use cmsis_native::CmsisNativeOperations;
use defmt::info;
use embassy_executor::SpawnToken;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use static_cell::StaticCell;
use synth_engine::{Q15, SynthEngine, wavetable::saw_wavetable::SAW_WAVETABLE};

use crate::midi_task::{MIDI_CHANNEL_SIZE, MIDI_TASK_CHANNEL};

#[cfg(feature = "audio-usb")]
use crate::audio_task::{SampleBlock, USB_MAX_SAMPLE_COUNT};
#[cfg(feature = "audio-usb")]
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
#[cfg(feature = "audio-usb")]
use embassy_sync::zerocopy_channel;

const VOICE_BANK_SIZE: usize = 16;

#[cfg(not(feature = "audio-usb"))]
const RUN_RATE_HZ: u16 = 1000;

#[cfg(feature = "audio-usb")]
const WINDOW_SIZE: usize = USB_MAX_SAMPLE_COUNT;

#[cfg(not(feature = "audio-usb"))]
const WINDOW_SIZE: usize = 128;

const ATTACK_CONFIG: u8 = 40;
const DECAY_RELEASE_CONFIG: u8 = 127;
const SUSTAIN_CONFIG: u8 = 200;

pub struct SynthEngineTaskState<'ch, 'wt> {
    synth_engine: SynthEngine<
        'ch,
        'wt,
        CriticalSectionRawMutex,
        MIDI_CHANNEL_SIZE,
        VOICE_BANK_SIZE,
        WINDOW_SIZE,
    >,
}

impl<'ch, 'wt> SynthEngineTaskState<'ch, 'wt> {
    pub fn new(
        synth_engine: SynthEngine<
            'ch,
            'wt,
            CriticalSectionRawMutex,
            MIDI_CHANNEL_SIZE,
            VOICE_BANK_SIZE,
            WINDOW_SIZE,
        >,
    ) -> SynthEngineTaskState<'ch, 'wt> {
        SynthEngineTaskState { synth_engine }
    }
}

pub static SYNTH_ENGINE_TASK_STATE: StaticCell<SynthEngineTaskState> = StaticCell::new();

#[cfg(feature = "audio-usb")]
pub fn create_task(
    audio_sender: zerocopy_channel::Sender<'static, NoopRawMutex, SampleBlock>,
) -> SpawnToken<impl Sized> {
    let synth_engine = SynthEngine::new(
        MIDI_TASK_CHANNEL.receiver(),
        &SAW_WAVETABLE,
        SUSTAIN_CONFIG,
        ATTACK_CONFIG,
        DECAY_RELEASE_CONFIG,
    );

    synth_engine_task(
        SYNTH_ENGINE_TASK_STATE.init(SynthEngineTaskState::new(synth_engine)),
        audio_sender,
    )
}

#[cfg(not(feature = "audio-usb"))]
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

#[cfg(feature = "audio-usb")]
#[embassy_executor::task]
pub async fn synth_engine_task(
    state: &'static mut SynthEngineTaskState<'static, 'static>,
    mut audio_sender: zerocopy_channel::Sender<'static, NoopRawMutex, SampleBlock>,
) {
    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    let mut counter: u32 = 0;

    info!("Synth Engine: Task starting, rendering at USB audio rate");

    loop {
        // This blocks until there's space in the channel,
        // and that space is only freed when the USB audio sends samples,
        // so this effecively syncs audio generation to the USB polling
        // (with a buffer of 2 polls, to guarantee data is ready immediately)
        let audio_buffer = audio_sender.send().await;

        state
            .synth_engine
            .render_samples::<CmsisNativeOperations>(&mut buffer);

        audio_buffer.copy_from_slice(&bytemuck::cast::<[Q15; WINDOW_SIZE], [i16; WINDOW_SIZE]>(
            buffer,
        ));

        audio_sender.send_done();

        if counter == 0 {
            info!("Voice bank state: {}", state.synth_engine.get_voice_bank());
        }

        counter = (counter + 1) % 1000;
    }
}

#[cfg(not(feature = "audio-usb"))]
#[embassy_executor::task]
pub async fn synth_engine_task(state: &'static mut SynthEngineTaskState<'static, 'static>) {
    use embassy_time::{Duration, Timer};

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    let mut counter: u16 = 0;

    info!("Synth Engine: Task starting at {} Hz", RUN_RATE_HZ);

    loop {
        state
            .synth_engine
            .render_samples::<CmsisNativeOperations>(&mut buffer);

        if counter == 0 {
            info!("Voice bank state: {}", state.synth_engine.get_voice_bank());
        }

        counter = (counter + 1) % RUN_RATE_HZ;

        Timer::after(Duration::from_hz(RUN_RATE_HZ.into())).await;
    }
}
