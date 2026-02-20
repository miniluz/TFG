#![cfg_attr(not(test), no_std)]

pub mod adsr;
pub mod capacitor;
pub mod generator;
#[cfg(feature = "octave-filter")]
pub mod octave_filter;
mod voice_bank;
pub mod wavetable;

/// Default number of samples to process in each render cycle
/// Used in tests and examples
pub const WINDOW_SIZE: usize = 128;

/// Sample rate in Hz
pub const SAMPLE_RATE: u32 = 48000;

use config::Config;
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Receiver, signal::Signal};
use midi::MidiEvent;

pub use cmsis_interface::{CmsisOperations, Q15};
pub use generator::Generator;
#[cfg(feature = "octave-filter")]
pub use octave_filter::OctaveFilterBank;
pub use voice_bank::{Note, PlayNoteResult, Velocity, VoiceBank, VoiceStage};

pub struct SynthEngine<
    'ch,
    'wt,
    'cfg,
    M: RawMutex,
    const CHANNEL_SIZE: usize,
    const VOICE_BANK_SIZE: usize,
    const WINDOW_SIZE: usize,
    const PAGE_AMOUNT: usize,
    const ENCODER_AMOUNT: usize,
    const OCTAVE_FILTER_FIRST_PAGE: usize,
> {
    generator: Generator<
        'ch,
        'wt,
        'cfg,
        M,
        CHANNEL_SIZE,
        VOICE_BANK_SIZE,
        WINDOW_SIZE,
        PAGE_AMOUNT,
        ENCODER_AMOUNT,
    >,
    #[cfg(feature = "octave-filter")]
    octave_filter: OctaveFilterBank,
    #[cfg_attr(not(feature = "octave-filter"), allow(dead_code))]
    config_signal: &'cfg Signal<M, Config<PAGE_AMOUNT, ENCODER_AMOUNT>>,
}

impl<
    'ch,
    'wt,
    'cfg,
    M: RawMutex,
    const CHANNEL_SIZE: usize,
    const VOICE_BANK_SIZE: usize,
    const WINDOW_SIZE: usize,
    const PAGE_AMOUNT: usize,
    const ENCODER_AMOUNT: usize,
    const OCTAVE_FILTER_FIRST_PAGE: usize,
>
    SynthEngine<
        'ch,
        'wt,
        'cfg,
        M,
        CHANNEL_SIZE,
        VOICE_BANK_SIZE,
        WINDOW_SIZE,
        PAGE_AMOUNT,
        ENCODER_AMOUNT,
        OCTAVE_FILTER_FIRST_PAGE,
    >
{
    pub fn new(
        receiver: Receiver<'ch, M, MidiEvent, CHANNEL_SIZE>,
        config_signal: &'cfg Signal<M, Config<PAGE_AMOUNT, ENCODER_AMOUNT>>,
    ) -> Self {
        Self {
            generator: Generator::new(receiver, config_signal),
            #[cfg(feature = "octave-filter")]
            octave_filter: OctaveFilterBank::new(),
            config_signal,
        }
    }

    pub fn get_voice_bank(&self) -> &VoiceBank<'wt, VOICE_BANK_SIZE> {
        self.generator.get_voice_bank()
    }

    #[cfg(feature = "octave-filter")]
    fn update_octave_filter_config(&mut self) {
        if let Some(config) = self.config_signal.try_take() {
            for band in 0..6 {
                let page_idx = OCTAVE_FILTER_FIRST_PAGE + (band / 3);
                let encoder_idx = band % 3;
                if page_idx < PAGE_AMOUNT && encoder_idx < ENCODER_AMOUNT {
                    let gain_value = config.pages[page_idx].values[encoder_idx];
                    self.octave_filter.set_band_gain(band, gain_value);
                }
            }
        }
    }

    pub fn render_samples<T: CmsisOperations>(&mut self, output_samples: &mut [Q15; WINDOW_SIZE]) {
        #[cfg(feature = "octave-filter")]
        self.update_octave_filter_config();

        let mut buffer = [Q15::ZERO; WINDOW_SIZE];
        self.generator.render_samples::<T>(&mut buffer);

        #[cfg(feature = "octave-filter")]
        {
            self.octave_filter
                .process::<T, WINDOW_SIZE>(&buffer, output_samples);
        }

        #[cfg(not(feature = "octave-filter"))]
        {
            output_samples.copy_from_slice(&buffer);
        }
    }
}
