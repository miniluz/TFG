use config::Config;
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Receiver};
use midi::MidiEvent;

pub use crate::voice_bank::{Note, PlayNoteResult, Velocity, VoiceBank, VoiceStage};
use crate::wavetable::{
    saw_wavetable::SAW_WAVETABLE, sine_wavetable::SINE_WAVETABLE,
    square_wavetable::SQUARE_WAVETABLE, triangle_wavetable::TRIANGLE_WAVETABLE,
};
pub use cmsis_interface::{CmsisOperations, Q15};

pub struct Generator<
    'ac,
    'wt,
    M: RawMutex,
    const CHANNEL_SIZE: usize,
    const VOICE_BANK_SIZE: usize,
    const WINDOW_SIZE: usize,
    const PAGE_AMOUNT: usize,
    const ENCODER_AMOUNT: usize,
> {
    voice_bank: VoiceBank<'wt, 'ac, M, VOICE_BANK_SIZE, CHANNEL_SIZE>,
}

impl<
    'ac,
    'wt,
    M: RawMutex,
    const CHANNEL_SIZE: usize,
    const VOICE_BANK_SIZE: usize,
    const WINDOW_SIZE: usize,
    const PAGE_AMOUNT: usize,
    const ENCODER_AMOUNT: usize,
> Generator<'ac, 'wt, M, CHANNEL_SIZE, VOICE_BANK_SIZE, WINDOW_SIZE, PAGE_AMOUNT, ENCODER_AMOUNT>
{
    const VOICE_BIT_SHIFT_SIZE: i8 = -((if VOICE_BANK_SIZE == 1 {
        0
    } else {
        (VOICE_BANK_SIZE - 1).ilog2() + 1
    }) as i8);

    pub fn get_wavetable_for_encoder(encoder: u8) -> &'static [Q15; 256] {
        match encoder % 4 {
            0 => &SINE_WAVETABLE,
            1 => &SAW_WAVETABLE,
            2 => &SQUARE_WAVETABLE,
            3 => &TRIANGLE_WAVETABLE,
            _ => &SINE_WAVETABLE,
        }
    }

    pub fn new(
        receiver: Receiver<'ac, M, MidiEvent, CHANNEL_SIZE>,
        initial_config: &Config<PAGE_AMOUNT, ENCODER_AMOUNT>,
    ) -> Self {
        let attack = initial_config.pages[0].values[0];
        let sustain = initial_config.pages[0].values[1];
        let decay_release = initial_config.pages[0].values[2];
        let osc_type = initial_config.pages[1].values[0] % 4;
        let wavetable = match osc_type {
            0 => &SINE_WAVETABLE,
            1 => &SAW_WAVETABLE,
            2 => &SQUARE_WAVETABLE,
            _ => &TRIANGLE_WAVETABLE,
        };

        Self {
            voice_bank: VoiceBank::new(wavetable, sustain, attack, decay_release, receiver),
        }
    }

    pub fn get_voice_bank(&self) -> &VoiceBank<'wt, 'ac, M, VOICE_BANK_SIZE, CHANNEL_SIZE> {
        &self.voice_bank
    }

    pub fn apply_config(&mut self, config: &Config<PAGE_AMOUNT, ENCODER_AMOUNT>) {
        let attack = config.pages[0].values[0];
        let sustain = config.pages[0].values[1];
        let decay_release = config.pages[0].values[2];

        self.voice_bank
            .set_adsr_config_all_voices(sustain, attack, decay_release);

        let osc_type = config.pages[1].values[0] % 4;
        let wavetable = Self::get_wavetable_for_encoder(osc_type);

        self.voice_bank.set_wavetable_all_voices(wavetable);
    }

    pub fn render_samples<T: CmsisOperations>(&mut self, sample_buffer: &mut [Q15]) {
        if sample_buffer.len() != WINDOW_SIZE {
            panic!();
        }

        self.voice_bank.process_midi_events();

        let mut output_buf = [Q15::ZERO; WINDOW_SIZE];

        let mut i: usize = 0;

        for voice in self.voice_bank.voices.iter_mut() {
            if voice.adsr.is_idle() {
                continue;
            }

            // Temporary buffers for this voice
            let mut wavetable_buf = [Q15::ZERO; WINDOW_SIZE];
            let mut envelope_buf = [Q15::ZERO; WINDOW_SIZE];
            let mut mixed_buf = [Q15::ZERO; WINDOW_SIZE];

            // Generate wavetable samples
            voice
                .wavetable_osc
                .get_samples::<T, WINDOW_SIZE>(&mut wavetable_buf);

            // Generate ADSR envelope (now includes velocity scaling)
            voice.adsr.get_samples::<WINDOW_SIZE>(&mut envelope_buf);

            // Multiply wavetable by envelope (element-wise)
            T::multiply_q15(&wavetable_buf, &envelope_buf, &mut mixed_buf);

            T::shift_in_place_q15(&mut mixed_buf, Self::VOICE_BIT_SHIFT_SIZE);

            if i.is_multiple_of(2) {
                // The first time we copy to sample_buffer, so it doesn't matter what it had inside
                T::add_q15(&output_buf, &mixed_buf, sample_buffer);
            } else {
                // Then we write to output_buf
                T::add_q15(sample_buffer, &mixed_buf, &mut output_buf);
            }

            i += 1;
        }

        if i.is_multiple_of(2) {
            // If it's a multiple of two, that means the last time it was odd and copied to
            // output_buf (then we added one, which is why it's even). It means the latest value is
            // at output_buf and we need to copy it.
            sample_buffer.copy_from_slice(&output_buf);
        }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn get_voice_bank_mut(&mut self) -> &mut VoiceBank<'wt, 'ac, M, VOICE_BANK_SIZE, CHANNEL_SIZE> {
        &mut self.voice_bank
    }
}

#[cfg(test)]
mod test;
