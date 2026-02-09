#![cfg_attr(not(test), no_std)]

pub mod adsr;
pub mod capacitor;
mod voice_bank;
pub mod wavetable;

/// Default number of samples to process in each render cycle
/// Used in tests and examples
pub const WINDOW_SIZE: usize = 128;

/// Sample rate in Hz
pub const SAMPLE_RATE: u32 = 48000;

use config::Config;
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Receiver, signal::Signal};
use heapless::Deque;
use midi::MidiEvent;

pub use cmsis_interface::{CmsisOperations, Q15};
pub use voice_bank::{Note, PlayNoteResult, Velocity, VoiceBank, VoiceStage};

use crate::wavetable::{saw_wavetable::SAW_WAVETABLE, sine_wavetable::SINE_WAVETABLE, square_wavetable::SQUARE_WAVETABLE, triangle_wavetable::TRIANGLE_WAVETABLE};

#[derive(Debug, Clone, Copy)]
struct PendingNote {
    note: Note,
    velocity: Velocity,
}


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
> {
    voice_bank: VoiceBank<'wt, VOICE_BANK_SIZE>,
    receiver: Receiver<'ch, M, MidiEvent, CHANNEL_SIZE>,
    note_queue: Deque<PendingNote, VOICE_BANK_SIZE>,
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
> SynthEngine<'ch, 'wt, 'cfg, M, CHANNEL_SIZE, VOICE_BANK_SIZE, WINDOW_SIZE, PAGE_AMOUNT, ENCODER_AMOUNT>
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
        receiver: Receiver<'ch, M, MidiEvent, CHANNEL_SIZE>,
        config_signal: &'cfg Signal<M, Config<PAGE_AMOUNT, ENCODER_AMOUNT>>,
    ) -> Self {
        // Get initial config from signal or use defaults
        let initial_config = config_signal.try_take();

        let (sustain, attack, decay_release, initial_wavetable) = if let Some(config) = initial_config {
            let sustain = config.pages[0].values[1];
            let attack = config.pages[0].values[0];
            let decay_release = config.pages[0].values[2];
            let osc_type = config.pages[1].values[0] % 4;
            let wavetable = match osc_type {
                0 => &SINE_WAVETABLE,
                1 => &SAW_WAVETABLE,
                2 => &SQUARE_WAVETABLE,
                _ => &TRIANGLE_WAVETABLE
            };
            (sustain, attack, decay_release, wavetable)
        } else {
            // Default values
            (200, 50, 100, &SINE_WAVETABLE)
        };

        Self {
            voice_bank: VoiceBank::new(
                initial_wavetable,
                sustain,
                attack,
                decay_release,
            ),
            receiver,
            note_queue: Deque::new(),
            config_signal,
        }
    }

    pub fn get_voice_bank(&self) -> &VoiceBank<'wt, VOICE_BANK_SIZE> {
        &self.voice_bank
    }

    fn check_and_apply_config_updates(&mut self) {
        // Non-blocking check for config updates
        if let Some(config) = self.config_signal.try_take() {
            // Page 0: ADSR configuration
            let attack = config.pages[0].values[0];
            let sustain = config.pages[0].values[1];
            let decay_release = config.pages[0].values[2];

            self.voice_bank.set_adsr_config_all_voices(sustain, attack, decay_release);

            // Page 1: Oscillator type (modulo 4)
            let osc_type = config.pages[1].values[0] % 4;
            let wavetable = Self::get_wavetable_for_encoder(osc_type);

            self.voice_bank.set_wavetable_all_voices(wavetable);
        }
    }

    pub fn render_samples<T: CmsisOperations>(&mut self, sample_buffer: &mut [Q15]) {
        if sample_buffer.len() != WINDOW_SIZE {
            panic!();
        }

        // Phase 0: Check for config updates
        self.check_and_apply_config_updates();

        // Phase 1: Process incoming MIDI events
        while let Ok(event) = self.receiver.try_receive() {
            match event {
                MidiEvent::NoteOff { key, vel: _ } => {
                    self.voice_bank.release_note(key.into());
                    self.note_queue
                        .retain(|PendingNote { note, velocity: _ }| note.as_u8() != key);
                }
                MidiEvent::NoteOn { key, vel } => {
                    let pending = PendingNote {
                        note: key.into(),
                        velocity: vel.into(),
                    };
                    // Add, dropping oldest
                    if self
                        .note_queue
                        .iter()
                        .all(|PendingNote { note, velocity: _ }| note.as_u8() != key)
                    {
                        let _ = self.note_queue.push_back(pending);
                    }
                }
            }
        }

        // Phase 2: Process note queue
        while let Some(&pending) = self.note_queue.front() {
            match self.voice_bank.play_note(pending.note, pending.velocity) {
                PlayNoteResult::Success => {
                    // Successfully allocated voice (retriggered or found idle voice)
                    self.note_queue.pop_front();
                }
                PlayNoteResult::AllVoicesBusy => {
                    // No idle voice - quick_release one and leave note queued
                    // Only call quick_release if we have more queued notes than voices being released
                    let queue_count = self.note_queue.len();
                    let quick_release_count = self.voice_bank.count_voices_in_quick_release();

                    if queue_count > quick_release_count {
                        self.voice_bank.quick_release();
                    }
                    break;
                }
            }
        }

        // Zero the output buffer using CMSIS
        let zero_buf = [Q15::ZERO; WINDOW_SIZE];
        let mut output_buf = [Q15::ZERO; WINDOW_SIZE];
        sample_buffer.copy_from_slice(&zero_buf);

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

            // Accumulate into output buffer using CMSIS add
            output_buf.copy_from_slice(sample_buffer);
            let mut result_buf = [Q15::ZERO; WINDOW_SIZE];
            T::add_q15(&output_buf, &mixed_buf, &mut result_buf);
            sample_buffer.copy_from_slice(&result_buf);
        }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn get_voice_bank_mut(&mut self) -> &mut VoiceBank<'wt, VOICE_BANK_SIZE> {
        &mut self.voice_bank
    }
}
#[cfg(test)]
mod test;
