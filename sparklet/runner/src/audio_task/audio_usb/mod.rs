pub mod hardware;

use core::sync::atomic::{AtomicI16, Ordering};
use defmt::info;
use embassy_executor::SpawnToken;
use embassy_stm32::peripherals;
use embassy_stm32::usb;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::zerocopy_channel;
use embassy_usb::class::uac1::microphone::{self, Volume};
use embassy_usb::driver::EndpointError;
use static_cell::StaticCell;

use hardware::AudioUsbHardware;

pub use hardware::{AUDIO_CHANNELS, USB_MAX_PACKET_SIZE, USB_MAX_SAMPLE_COUNT};


/// Shared volume state accessible from both tasks
struct VolumeState {
    current_volume_8q8: AtomicI16,
}

static VOLUME_STATE: StaticCell<VolumeState> = StaticCell::new();

/// Applies volume gain to a sample using shift-based approximation.
///
/// Note: This uses LINEAR scaling which is NOT correct for audio (should be exponential),
/// but provides a simple demonstration. Every -6 dB H halves the amplitude.
#[inline]
fn apply_volume_gain(sample: i16, volume_8q8: i16) -> i16 {
    // Convert 8.8 fixed point to integer dB
    let db = volume_8q8 / 256;

    // Calculate right shifts needed (every -6 dB = 1 bit shift)
    let shifts = (-db) / 6;

    if shifts >= 15 {
        0 // Essentially silent
    } else if shifts <= 0 {
        sample // 0 dB or positive
    } else {
        sample >> shifts
    }
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => defmt::panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

/// Handles streaming of audio data to the host.
async fn stream_handler<'d, T: usb::Instance + 'd>(
    stream: &mut microphone::Stream<'d, usb::Driver<'d, T>>,
    receiver: &mut zerocopy_channel::Receiver<'static, NoopRawMutex, super::SampleBlock>,
    volume_state: &'static VolumeState,
) -> Result<(), Disconnected> {
    info!("USB Audio: Stream handler starting...");
    let mut usb_data = [0u8; USB_MAX_PACKET_SIZE];
    let mut packet_count = 0u32;

    loop {
        let samples = receiver.receive().await;

        // Get current volume
        let volume_8q8 = volume_state.current_volume_8q8.load(Ordering::Relaxed);

        // Apply volume gain to i16 samples and convert to bytes
        let mut scaled_samples = [0i16; USB_MAX_SAMPLE_COUNT];
        for (i, &sample) in samples.iter().enumerate() {
            scaled_samples[i] = apply_volume_gain(sample, volume_8q8);
        }

        usb_data.copy_from_slice(&bytemuck::cast::<
            [i16; USB_MAX_SAMPLE_COUNT],
            [u8; USB_MAX_PACKET_SIZE],
        >(scaled_samples));

        receiver.receive_done();

        stream.write_packet(&usb_data).await?;

        packet_count += 1;
        if packet_count % 1000 == 0 {
            info!("USB Audio: Streamed {} packets", packet_count);
        }
    }
}


/// Sends audio samples to the host.
#[embassy_executor::task]
async fn usb_streaming_task(
    mut stream: microphone::Stream<'static, usb::Driver<'static, peripherals::USB_OTG_HS>>,
    mut receiver: zerocopy_channel::Receiver<'static, NoopRawMutex, super::SampleBlock>,
    volume_state: &'static VolumeState,
) {
    loop {
        stream.wait_connection().await;
        info!("USB Audio: Connected - microphone streaming active");

        _ = stream_handler(&mut stream, &mut receiver, volume_state).await;

        info!("USB Audio: Disconnected");
    }
}

/// Checks for changes on the control monitor of the class.
#[embassy_executor::task]
async fn usb_control_task(
    control_monitor: microphone::ControlMonitor<'static>,
    volume_state: &'static VolumeState,
) {
    info!("USB Audio: Control task starting...");
    loop {
        control_monitor.changed().await;

        for channel in AUDIO_CHANNELS {
            match control_monitor.gain(channel).unwrap() {
                Volume::Muted => {
                    info!("USB Audio: Channel {} muted", channel);
                    volume_state
                        .current_volume_8q8
                        .store(-25600, Ordering::Relaxed);
                }
                Volume::DeciBel(vol_8q8) => {
                    let db_int = vol_8q8 / 256;
                    info!(
                        "USB Audio: Channel {} gain: {} dB (raw: {})",
                        channel, db_int, vol_8q8
                    );
                    volume_state.current_volume_8q8.store(vol_8q8, Ordering::Relaxed);
                }
            }
        }

        let sample_rate = control_monitor.sample_rate_hz();
        info!("USB Audio: Sample rate: {} Hz", sample_rate);
    }
}

pub fn create_audio_tasks(
    audio_hardware: AudioUsbHardware<'static>,
) -> (
    SpawnToken<impl Sized>,
    SpawnToken<impl Sized>,
    zerocopy_channel::Sender<'static, NoopRawMutex, super::SampleBlock>,
) {
    info!("USB Audio: Creating tasks...");

    // Initialize volume state (start at 0 dB = maximum volume)
    let volume_state = VOLUME_STATE.init(VolumeState {
        current_volume_8q8: AtomicI16::new(0),
    });

    // Initialize zero-copy channel for audio samples
    let sample_blocks =
        super::SAMPLE_BLOCKS.init([[0; super::USB_MAX_SAMPLE_COUNT]; 2]);
    let channel = super::AUDIO_CHANNEL.init(zerocopy_channel::Channel::new(sample_blocks));
    let (sender, receiver) = channel.split();

    // Create spawn tokens for the two USB audio tasks
    let control_task = usb_control_task(audio_hardware.control_monitor, volume_state);
    let streaming_task = usb_streaming_task(audio_hardware.stream, receiver, volume_state);

    info!("USB Audio: Tasks created successfully");

    (control_task, streaming_task, sender)
}
