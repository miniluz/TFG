#![cfg_attr(not(test), no_std)]

use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Sender};
use midly::{MidiMessage, live::LiveEvent, num::u7, stream::MidiStream};

pub enum MidiEvent {
    NoteOff { key: u7, vel: u7 },
    NoteOn { key: u7, vel: u7 },
}

pub struct MidiReceiver<'ch, M: RawMutex, const N: usize> {
    sender: Sender<'ch, M, MidiEvent, N>,
    midi_stream: MidiStream,
}

impl<'ch, M: RawMutex, const N: usize> MidiReceiver<'ch, M, N> {
    pub fn new(sender: Sender<'ch, M, MidiEvent, N>) -> Self {
        let midi_stream = MidiStream::new();

        MidiReceiver {
            sender,
            midi_stream,
        }
    }

    fn handle_event(sender: &Sender<'ch, M, MidiEvent, N>, event: LiveEvent<'_>) {
        if let LiveEvent::Midi {
            channel: _,
            message,
        } = event
        {
            let event_to_add: MidiEvent = match message {
                MidiMessage::NoteOff { key, vel } => MidiEvent::NoteOff { key, vel },
                MidiMessage::NoteOn { key, vel } => MidiEvent::NoteOn { key, vel },
                _ => return,
            };

            // only fails if full. if full, the message should be discareded anyways
            sender.try_send(event_to_add).ok();
        }
    }

    pub async fn process_bytes(&mut self, bytes: &[u8]) {
        self.midi_stream
            .feed(bytes, |event| Self::handle_event(&self.sender, event));
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
