#![cfg_attr(not(test), no_std)]

use defmt::Format;
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Receiver};
use midi::MidiEvent;

pub use cmsis_interface::Q15;

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceStage {
    Free,
    Held,
}

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
struct Voice {
    start: u64,
    note: u8,
    velocity: u8,
    stage: VoiceStage,
}

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceBank<const N: usize> {
    voices: [Voice; N],
}

impl<const N: usize> VoiceBank<N> {
    pub fn new() -> Self {
        Self {
            voices: [Voice {
                start: 0,
                note: 0,
                velocity: 0,
                stage: VoiceStage::Free,
            }; N],
        }
    }

    pub fn process_midi_event(&mut self, event: MidiEvent) {
        match event {
            MidiEvent::NoteOn { key, vel } => self.play_note(key, vel),
            MidiEvent::NoteOff { key, vel: _ } => self.release_note(key),
        }
    }

    pub fn play_note(&mut self, note: u8, velocity: u8) {
        let mut earliest_start_index: usize = 0;
        let mut earliest_start_value: u64 = 0;

        for (index, voice) in self.voices.iter_mut().enumerate() {
            match voice.stage {
                VoiceStage::Free => {
                    voice.note = note;
                    voice.velocity = velocity;
                    voice.stage = VoiceStage::Held;
                    return;
                }
                VoiceStage::Held => {
                    if voice.start < earliest_start_value {
                        earliest_start_index = index;
                        earliest_start_value = voice.start;
                    }
                }
            }
        }

        // No free note found
        let earliest_voice = &mut self.voices[earliest_start_index];
        earliest_voice.start = 0;
        earliest_voice.note = note;
        earliest_voice.velocity = velocity;
        earliest_voice.stage = VoiceStage::Held;
    }

    pub fn release_note(&mut self, note: u8) {
        for voice in self.voices.iter_mut() {
            if voice.note == note {
                voice.stage = VoiceStage::Free;
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn count_active_voices(&self) -> usize {
        self.voices
            .iter()
            .filter(|v| v.stage == VoiceStage::Held)
            .count()
    }

    #[cfg(test)]
    pub(crate) fn get_voice_state(&self, index: usize) -> Voice {
        self.voices[index]
    }
}

pub struct SynthEngine<'ch, M: RawMutex, const CHANNEL_SIZE: usize, const VOICE_BANK_SIZE: usize> {
    voice_bank: VoiceBank<VOICE_BANK_SIZE>,
    receiver: Receiver<'ch, M, MidiEvent, CHANNEL_SIZE>,
}

impl<'ch, M: RawMutex, const CHANNEL_SIZE: usize, const VOICE_BANK_SIZE: usize>
    SynthEngine<'ch, M, CHANNEL_SIZE, VOICE_BANK_SIZE>
{
    pub fn new(receiver: Receiver<'ch, M, MidiEvent, CHANNEL_SIZE>) -> Self {
        Self {
            voice_bank: VoiceBank::new(),
            receiver,
        }
    }

    pub fn get_voice_bank(&self) -> &VoiceBank<VOICE_BANK_SIZE> {
        &self.voice_bank
    }

    pub fn render_samples(&mut self, sample_buffer: &mut [Q15]) {
        while let Ok(event) = self.receiver.try_receive() {
            self.voice_bank.process_midi_event(event);
        }

        sample_buffer.iter_mut().for_each(|s| *s = Q15::ZERO);
    }

    #[cfg(test)]
    pub(crate) fn get_voice_bank_mut(&mut self) -> &mut VoiceBank<VOICE_BANK_SIZE> {
        &mut self.voice_bank
    }
}
#[cfg(test)]
mod test;
