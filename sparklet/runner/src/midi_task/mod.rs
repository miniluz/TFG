use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use midi::MidiEvent;

#[cfg(feature = "midi-din")]
pub mod midi_din;
#[cfg(feature = "midi-din")]
pub use midi_din::create_midi_task;

#[cfg(feature = "midi-usb")]
pub mod midi_usb;
#[cfg(feature = "midi-usb")]
pub use midi_usb::create_midi_task;

#[cfg(all(feature = "midi-usb", feature = "midi-din"))]
compile_error!("feature \"midi-usb\" and feature \"midi-din\" cannot be enabled at the same time");

pub const MIDI_CHANNEL_SIZE: usize = 16;

pub static MIDI_TASK_CHANNEL: Channel<CriticalSectionRawMutex, MidiEvent, MIDI_CHANNEL_SIZE> =
    Channel::new();
