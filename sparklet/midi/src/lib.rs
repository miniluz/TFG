#![cfg_attr(not(test), no_std)]

use defmt::{Format, info};
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Sender};
use midly::{MidiMessage, live::LiveEvent, stream::MidiStream};

pub use midly::num::u7;

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiEvent {
    NoteOff { key: u8, vel: u8 },
    NoteOn { key: u8, vel: u8 },
}

pub struct MidiListener<'ch, M: RawMutex, const N: usize> {
    sender: Sender<'ch, M, MidiEvent, N>,
    midi_stream: MidiStream<MidiListenerBuffer>,
}

midly::stack_buffer! {
    struct MidiListenerBuffer([u8; 4]);
}

impl<'ch, M: RawMutex, const N: usize> MidiListener<'ch, M, N> {
    pub fn new(sender: Sender<'ch, M, MidiEvent, N>) -> Self {
        let midi_stream = MidiStream::with_buffer(MidiListenerBuffer::new());

        MidiListener {
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
                MidiMessage::NoteOff { key, vel } => MidiEvent::NoteOff {
                    key: key.into(),
                    vel: vel.into(),
                },
                MidiMessage::NoteOn { key, vel } => MidiEvent::NoteOn {
                    key: key.into(),
                    vel: vel.into(),
                },
                _ => return,
            };

            info!("Adding event: {:#?}", event_to_add);

            // only fails if full. if full, the message should be discareded anyways
            sender.try_send(event_to_add).ok();
        }
    }

    pub fn process_bytes(&mut self, bytes: &[u8]) {
        self.midi_stream
            .feed(bytes, |event| Self::handle_event(&self.sender, event));
    }
}

#[cfg(test)]
mod test;
