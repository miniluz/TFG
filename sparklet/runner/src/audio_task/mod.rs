#[cfg(feature = "audio-usb")]
pub mod audio_usb;

#[cfg(feature = "audio-usb")]
pub use audio_usb::create_audio_tasks;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::zerocopy_channel;
use static_cell::StaticCell;

// Audio sample block type - i16 samples for audio processing
pub const USB_MAX_SAMPLE_COUNT: usize = 48; // 48kHz * 1 channel / 1000ms
pub type SampleBlock = [i16; USB_MAX_SAMPLE_COUNT];

// Shared audio channel for transferring samples between synth engine and USB streaming tasks
pub static AUDIO_CHANNEL: StaticCell<zerocopy_channel::Channel<'static, NoopRawMutex, SampleBlock>> =
    StaticCell::new();
pub static SAMPLE_BLOCKS: StaticCell<[SampleBlock; 2]> = StaticCell::new();
