#![cfg_attr(not(test), no_std)]

pub mod adsr;
mod voice_bank;
mod wavetable;

/// Number of samples to process in each render cycle
pub const WINDOW_SIZE: usize = 128;

/// Sample rate in Hz
pub const SAMPLE_RATE: u32 = 48000;

use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Receiver};
use heapless::Deque;
use midi::MidiEvent;

pub use cmsis_interface::{CmsisOperations, Q15};
pub use voice_bank::{Note, Velocity, VoiceBank, VoiceStage};

#[derive(Debug, Clone, Copy)]
struct PendingNote {
    note: Note,
    velocity: Velocity,
}

pub struct SynthEngine<
    'ch,
    'wt,
    M: RawMutex,
    const CHANNEL_SIZE: usize,
    const VOICE_BANK_SIZE: usize,
> {
    voice_bank: VoiceBank<'wt, VOICE_BANK_SIZE>,
    receiver: Receiver<'ch, M, MidiEvent, CHANNEL_SIZE>,
    note_queue: Deque<PendingNote, VOICE_BANK_SIZE>,
}

impl<'ch, 'wt, M: RawMutex, const CHANNEL_SIZE: usize, const VOICE_BANK_SIZE: usize>
    SynthEngine<'ch, 'wt, M, CHANNEL_SIZE, VOICE_BANK_SIZE>
{
    const VOICE_BIT_SHIFT_SIZE: i8 = (if VOICE_BANK_SIZE == 1 {
        0
    } else {
        (VOICE_BANK_SIZE - 1).ilog2() + 1
    }) as i8;

    pub fn new(
        receiver: Receiver<'ch, M, MidiEvent, CHANNEL_SIZE>,
        wavetable: &'wt [Q15; 256],
        sustain_config: u8,
        attack_config: u8,
        decay_release_config: u8,
    ) -> Self {
        Self {
            voice_bank: VoiceBank::new(
                wavetable,
                sustain_config,
                attack_config,
                decay_release_config,
            ),
            receiver,
            note_queue: Deque::new(),
        }
    }

    pub fn get_voice_bank(&self) -> &VoiceBank<'wt, VOICE_BANK_SIZE> {
        &self.voice_bank
    }

    pub fn render_samples<T: CmsisOperations>(&mut self, sample_buffer: &mut [Q15]) {
        if sample_buffer.len() != WINDOW_SIZE {
            panic!();
        }

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
            // Try to find an idle voice
            if self.find_idle_voice().is_some() {
                // Assign note to idle voice
                self.voice_bank.play_note(pending.note, pending.velocity);
                self.note_queue.pop_front();
            } else {
                // No idle voice - find one to quick_release
                if let Some(voice_index) = self.find_voice_to_release() {
                    self.quick_release_voice(voice_index);
                }
                // Leave note in queue for next render cycle
                break;
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
            let mut velocity_scaled_buf = [Q15::ZERO; WINDOW_SIZE];

            // Generate wavetable samples
            voice
                .wavetable_osc
                .get_samples::<T, WINDOW_SIZE>(&mut wavetable_buf);

            // Generate ADSR envelope
            voice.adsr.get_samples::<WINDOW_SIZE>(&mut envelope_buf);

            // Multiply wavetable by envelope (element-wise)
            T::multiply_q15(&wavetable_buf, &envelope_buf, &mut mixed_buf);

            // Scale by velocity (0-127 -> Q15 scale factor)
            // Velocity 127 = 1.0, velocity 64 ~= 0.5
            let velocity_scale = Q15::from_bits((voice.velocity.as_u8() as i16) << 8);
            let velocity_array = [velocity_scale; WINDOW_SIZE];
            T::multiply_q15(&mixed_buf, &velocity_array, &mut velocity_scaled_buf);

            T::shift_in_place_q15(&mut velocity_scaled_buf, Self::VOICE_BIT_SHIFT_SIZE);

            // Accumulate into output buffer using CMSIS add
            output_buf.copy_from_slice(sample_buffer);
            let mut result_buf = [Q15::ZERO; WINDOW_SIZE];
            T::add_q15(&output_buf, &velocity_scaled_buf, &mut result_buf);
            sample_buffer.copy_from_slice(&result_buf);
        }
    }

    fn find_idle_voice(&self) -> Option<usize> {
        self.voice_bank.voices.iter().position(|v| v.adsr.is_idle())
    }

    fn find_voice_to_release(&self) -> Option<usize> {
        // Priority 1: Find quietest voice in Release (not QuickRelease)
        if let Some(index) = self
            .voice_bank
            .voices
            .iter()
            .enumerate()
            .filter(|(_, v)| v.adsr.is_in_release())
            .min_by_key(|(_, v)| v.adsr.output)
            .map(|(index, _)| index)
        {
            return Some(index);
        }

        // Priority 2: Find oldest voice that's not in QuickRelease
        self.voice_bank
            .voices
            .iter()
            .enumerate()
            .filter(|(_, v)| !v.adsr.is_in_quick_release() && !v.adsr.is_idle())
            .min_by_key(|(_, v)| v.timestamp)
            .map(|(index, _)| index)
    }

    fn quick_release_voice(&mut self, index: usize) {
        self.voice_bank.voices[index].adsr.quick_release();
    }

    #[cfg(test)]
    pub(crate) fn get_voice_bank_mut(&mut self) -> &mut VoiceBank<'wt, VOICE_BANK_SIZE> {
        &mut self.voice_bank
    }
}
#[cfg(test)]
mod test;
