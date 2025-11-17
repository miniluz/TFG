#![cfg_attr(not(test), no_std)]

mod voice_bank;
mod wavetable;

use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Receiver};
use midi::MidiEvent;

pub use cmsis_interface::Q15;
pub use voice_bank::{Note, Velocity, VoiceBank, VoiceStage};

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
