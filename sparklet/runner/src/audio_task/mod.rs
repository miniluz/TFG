#[cfg(feature = "audio-usb")]
pub mod audio_usb;

#[cfg(feature = "audio-usb")]
pub use audio_usb::create_audio_tasks;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::zerocopy_channel;
use static_cell::StaticCell;

// Audio sample block type - size depends on USB configuration
pub const USB_MAX_PACKET_SIZE: usize = 96; // 48kHz * 1 channel * 2 bytes / 1000ms
pub type SampleBlock = [u8; USB_MAX_PACKET_SIZE];

// Shared audio channel for transferring samples between generator and streaming tasks
pub static AUDIO_CHANNEL: StaticCell<zerocopy_channel::Channel<'static, NoopRawMutex, SampleBlock>> =
    StaticCell::new();
pub static SAMPLE_BLOCKS: StaticCell<[SampleBlock; 2]> = StaticCell::new();
