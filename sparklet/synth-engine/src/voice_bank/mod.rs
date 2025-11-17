use defmt::Format;
use midi::MidiEvent;

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

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceStage {
    Free,
    Held,
}

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Voice {
    start: u64,
    note: Note,
    velocity: Velocity,
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
                note: Note(0),
                velocity: Velocity(0),
                stage: VoiceStage::Free,
            }; N],
        }
    }

    pub fn process_midi_event(&mut self, event: MidiEvent) {
        match event {
            MidiEvent::NoteOn { key, vel } => self.play_note(key.into(), vel.into()),
            MidiEvent::NoteOff { key, vel: _ } => self.release_note(key.into()),
        }
    }

    pub fn play_note(&mut self, note: Note, velocity: Velocity) {
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

    pub fn release_note(&mut self, note: Note) {
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
    pub(crate) fn get_voice_note(&self, index: usize) -> Note {
        self.voices[index].note
    }

    #[cfg(test)]
    pub(crate) fn get_voice_velocity(&self, index: usize) -> Velocity {
        self.voices[index].velocity
    }

    #[cfg(test)]
    pub(crate) fn get_voice_stage(&self, index: usize) -> VoiceStage {
        self.voices[index].stage
    }
}

#[cfg(test)]
mod test;
