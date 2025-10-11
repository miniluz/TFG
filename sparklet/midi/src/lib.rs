#![cfg_attr(not(test), no_std)]

use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Sender};
use midly::{MidiMessage, live::LiveEvent, stream::MidiStream};

pub use midly::num::u7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiEvent {
    NoteOff { key: u7, vel: u7 },
    NoteOn { key: u7, vel: u7 },
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
                MidiMessage::NoteOff { key, vel } => MidiEvent::NoteOff { key, vel },
                MidiMessage::NoteOn { key, vel } => MidiEvent::NoteOn { key, vel },
                _ => return,
            };

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
mod test {
    use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
    use midly::{
        MidiMessage,
        live::{LiveEvent, SystemCommon},
    };
    use pretty_assertions::assert_eq;

    use crate::{MidiEvent, MidiListener};

    macro_rules! setup {
        ($receiver:ident, $midi_listener:ident) => {
            let channel = Channel::<NoopRawMutex, MidiEvent, 4>::new();
            let sender = channel.sender();
            let $receiver = channel.receiver();
            let mut $midi_listener = MidiListener::new(sender);
        };
    }

    macro_rules! note_on {
        ($channel:expr, $key:expr, $vel:expr) => {
            LiveEvent::Midi {
                channel: $channel.into(),
                message: MidiMessage::NoteOn {
                    key: $key.into(),
                    vel: $vel.into(),
                },
            }
        };
    }

    macro_rules! note_off {
        ($channel:expr, $key:expr, $vel:expr) => {
            LiveEvent::Midi {
                channel: $channel.into(),
                message: MidiMessage::NoteOff {
                    key: $key.into(),
                    vel: $vel.into(),
                },
            }
        };
    }

    #[test]
    fn when_overflowing_it_discards_the_overflow() {
        setup!(receiver, midi_listener);

        let sample_midi = [
            note_on!(0, 0, 0),
            note_off!(0, 1, 1),
            note_on!(0, 2, 2),
            note_off!(0, 3, 3),
            note_on!(0, 4, 4),
            note_off!(0, 5, 5),
        ];

        let mut input_buffer: Vec<u8> = Vec::new();

        sample_midi
            .iter()
            .for_each(|ev| ev.write(&mut input_buffer).unwrap());

        midi_listener.process_bytes(&input_buffer);

        let mut output_buffer: Vec<MidiEvent> = Vec::new();

        while let Ok(event) = receiver.try_receive() {
            output_buffer.push(event);
        }

        assert_eq!(
            output_buffer.as_slice(),
            &[
                MidiEvent::NoteOn {
                    key: 0.into(),
                    vel: 0.into()
                },
                MidiEvent::NoteOff {
                    key: 1.into(),
                    vel: 1.into()
                },
                MidiEvent::NoteOn {
                    key: 2.into(),
                    vel: 2.into()
                },
                MidiEvent::NoteOff {
                    key: 3.into(),
                    vel: 3.into()
                },
            ]
        );
    }

    #[test]
    fn when_receiving_from_multiple_channels_it_processes_all_of_them() {
        setup!(receiver, midi_listener);

        let sample_midi = [
            note_on!(0, 0, 0),
            note_off!(1, 1, 1),
            note_on!(2, 2, 2),
            note_off!(3, 3, 3),
        ];

        let mut input_buffer: Vec<u8> = Vec::new();

        sample_midi
            .iter()
            .for_each(|ev| ev.write(&mut input_buffer).unwrap());

        midi_listener.process_bytes(&input_buffer);

        let mut output_buffer: Vec<MidiEvent> = Vec::new();

        while let Ok(event) = receiver.try_receive() {
            output_buffer.push(event);
        }

        assert_eq!(
            output_buffer.as_slice(),
            &[
                MidiEvent::NoteOn {
                    key: 0.into(),
                    vel: 0.into()
                },
                MidiEvent::NoteOff {
                    key: 1.into(),
                    vel: 1.into()
                },
                MidiEvent::NoteOn {
                    key: 2.into(),
                    vel: 2.into()
                },
                MidiEvent::NoteOff {
                    key: 3.into(),
                    vel: 3.into()
                },
            ]
        );
    }

    #[test]
    fn when_receiving_garbage_it_processes_the_midi() {
        setup!(receiver, midi_listener);

        let sysex_contents = [8.into()].repeat(1000);
        let sysex = LiveEvent::Common(SystemCommon::SysEx(sysex_contents.as_slice()));

        let sample_midi = [
            note_on!(0, 0, 0),
            note_off!(1, 1, 1),
            sysex,
            note_on!(2, 2, 2),
            note_off!(3, 3, 3),
        ];

        let mut input_buffer: Vec<u8> = Vec::new();

        sample_midi[0..2]
            .iter()
            .for_each(|ev| ev.write(&mut input_buffer).unwrap());

        // add random data
        input_buffer
            .append(&mut [0x90, 0xf1, 0x56, 0x3e, 0xe3, 0x0d, 0x87, 0x78, 0xd1, 0xc4].repeat(1000));

        sample_midi[2..]
            .iter()
            .for_each(|ev| ev.write(&mut input_buffer).unwrap());

        midi_listener.process_bytes(&input_buffer);

        let mut output_buffer: Vec<MidiEvent> = Vec::new();

        while let Ok(event) = receiver.try_receive() {
            output_buffer.push(event);
        }

        assert_eq!(
            output_buffer.as_slice(),
            &[
                MidiEvent::NoteOn {
                    key: 0.into(),
                    vel: 0.into()
                },
                MidiEvent::NoteOff {
                    key: 1.into(),
                    vel: 1.into()
                },
                MidiEvent::NoteOn {
                    key: 2.into(),
                    vel: 2.into()
                },
                MidiEvent::NoteOff {
                    key: 3.into(),
                    vel: 3.into()
                },
            ]
        );
    }
}
