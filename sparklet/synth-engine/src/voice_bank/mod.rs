use defmt::Format;
use midi::MidiEvent;

use crate::{SAMPLE_RATE, adsr::ADSR, wavetable::WavetableOscillator};

/// A MIDI note number (0-127)
#[derive(Format, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Note(u8);

impl Note {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

impl From<u8> for Note {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Note> for u8 {
    fn from(note: Note) -> Self {
        note.0
    }
}

impl From<midi::u7> for Note {
    fn from(value: midi::u7) -> Self {
        Self(value.into())
    }
}

impl From<Note> for midi::u7 {
    fn from(note: Note) -> Self {
        midi::u7::from(note.0)
    }
}

/// A MIDI velocity (0-127)
#[derive(Format, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Velocity(u8);

impl Velocity {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

impl From<u8> for Velocity {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Velocity> for u8 {
    fn from(velocity: Velocity) -> Self {
        velocity.0
    }
}

impl From<midi::u7> for Velocity {
    fn from(value: midi::u7) -> Self {
        Self(value.into())
    }
}

impl From<Velocity> for midi::u7 {
    fn from(velocity: Velocity) -> Self {
        midi::u7::from(velocity.0)
    }
}

/// Result of attempting to play a note
#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayNoteResult {
    /// Successfully played the note (either retriggered or allocated a voice)
    Success,
    /// Could not play the note because all voices are busy
    AllVoicesBusy,
}

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceStage {
    Free,
    Held,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Voice<'a> {
    pub(crate) timestamp: u32,
    pub(crate) note: Note,
    pub(crate) velocity: Velocity,
    pub(crate) adsr: ADSR,
    pub(crate) wavetable_osc: WavetableOscillator<'a, SAMPLE_RATE>,
}

impl<'a> Voice<'a> {
    pub(crate) fn retrigger(&mut self, timestamp: u32, velocity: Velocity) {
        self.timestamp = timestamp;
        self.velocity = velocity;
        self.adsr.retrigger();
    }

    pub(crate) fn play_note(&mut self, timestamp: u32, note: Note, velocity: Velocity) {
        self.timestamp = timestamp;
        self.note = note;
        self.velocity = velocity;
        self.wavetable_osc.set_note(&note);
        self.adsr.play();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VoiceBank<'a, const N: usize> {
    pub(crate) voices: [Voice<'a>; N],
    pub(crate) timestamp_counter: u32,
}

impl<'a, const N: usize> VoiceBank<'a, N> {
    pub fn new(
        wavetable: &'a [cmsis_interface::Q15; 256],
        sustain_config: u8,
        attack_config: u8,
        decay_release_config: u8,
    ) -> Self {
        Self {
            voices: [Voice {
                timestamp: 0,
                note: Note(0),
                velocity: Velocity(0),
                adsr: ADSR::new(sustain_config, attack_config, decay_release_config),
                wavetable_osc: WavetableOscillator::new(wavetable),
            }; N],
            timestamp_counter: 0,
        }
    }

    pub fn process_midi_event(&mut self, event: MidiEvent) {
        match event {
            MidiEvent::NoteOn { key, vel } => {
                let _ = self.play_note(key.into(), vel.into());
            }
            MidiEvent::NoteOff { key, vel: _ } => {
                self.release_note(key.into());
            }
        }
    }

    pub fn play_note(&mut self, note: Note, velocity: Velocity) -> PlayNoteResult {
        self.play_note_optional_retrigger(note, velocity, true)
    }

    fn play_note_optional_retrigger(&mut self, note: Note, velocity: Velocity, retrigger: bool) -> PlayNoteResult {
        // Check for retriggering first
        if retrigger {
            for voice in self.voices.iter_mut() {
                if voice.note == note && !voice.adsr.is_idle() {
                    self.timestamp_counter = self.timestamp_counter.wrapping_add(1);
                    voice.retrigger(self.timestamp_counter, velocity);
                    return PlayNoteResult::Success;
                }
            }
        }

        // Find an idle voice
        for voice in self.voices.iter_mut() {
            if voice.adsr.is_idle() {
                self.timestamp_counter = self.timestamp_counter.wrapping_add(1);
                voice.play_note(self.timestamp_counter, note, velocity);
                return PlayNoteResult::Success;
            }
        }

        // No idle voice available
        PlayNoteResult::AllVoicesBusy
    }

    pub fn release_note(&mut self, note: Note) {
        for voice in self.voices.iter_mut() {
            if voice.note == note && !voice.adsr.is_idle() {
                voice.adsr.stop_playing();
            }
        }
    }

    pub fn quick_release(&mut self) {
        // Priority 1: Find quietest voice in Release (not QuickRelease)
        if let Some(index) = self
            .voices
            .iter()
            .enumerate()
            .filter(|(_, v)| v.adsr.is_in_release())
            .min_by_key(|(_, v)| v.adsr.output)
            .map(|(index, _)| index)
        {
            self.voices[index].adsr.quick_release();
            return;
        }

        // Priority 2: Find oldest voice that's not in QuickRelease and not idle
        if let Some(index) = self
            .voices
            .iter()
            .enumerate()
            .filter(|(_, v)| !v.adsr.is_in_quick_release() && !v.adsr.is_idle())
            .min_by_key(|(_, v)| v.timestamp)
            .map(|(index, _)| index)
        {
            self.voices[index].adsr.quick_release();
        }

        // If no voice found (all idle or in QuickRelease), this is a no-op
    }

    #[cfg(test)]
    pub(crate) fn play_duplicate_note(&mut self, note: Note, velocity: Velocity) -> PlayNoteResult {
        self.play_note_optional_retrigger(note, velocity, false)
    }

    #[cfg(test)]
    pub(crate) fn count_active_voices(&self) -> usize {
        self.voices.iter().filter(|v| !v.adsr.is_idle()).count()
    }

    #[cfg(test)]
    pub(crate) fn get_voice_note(&self, index: usize) -> Note {
        self.voices[index].note
    }

    #[cfg(test)]
    pub(crate) fn get_voice_velocity(&self, index: usize) -> Velocity {
        self.voices[index].velocity
    }

    #[cfg(test)]
    pub(crate) fn get_voice_stage(&self, index: usize) -> VoiceStage {
        // For backward compatibility with tests
        if self.voices[index].adsr.is_idle() {
            VoiceStage::Free
        } else {
            VoiceStage::Held
        }
    }
}

#[cfg(test)]
mod test;
