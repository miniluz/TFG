pub mod hardware;

use cmsis_native::CmsisNativeOperations;
use core::sync::atomic::{AtomicI16, Ordering};
use defmt::info;
use embassy_executor::SpawnToken;
use embassy_stm32::peripherals;
use embassy_stm32::usb;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::zerocopy_channel;
use embassy_usb::class::uac1::microphone::{self, Volume};
use embassy_usb::driver::EndpointError;
use fixed::traits::LossyInto;
use static_cell::StaticCell;
use synth_engine::Q15;
use synth_engine::adsr::db_linear_amplitude_table::DB_LINEAR_AMPLITUDE_TABLE;

use hardware::AudioUsbHardware;

pub use hardware::{AUDIO_CHANNELS, USB_MAX_PACKET_SIZE, USB_MAX_SAMPLE_COUNT};
use synth_engine::CmsisOperations;

/// Shared volume state accessible from both tasks
struct VolumeState {
    volume_mult: AtomicI16,
}

fn volume_8q8_to_mult(volume_8q8: i16) -> Q15 {
    // I want to map the (-80 * 256)..0 range to 0..255
    // x' = (x - min) / (max - min) * (max' - min') + min'
    // x' = (x + 80 * 256) / (80 * 256) * (255) + 0
    let index = (volume_8q8.clamp(-80 * 256, 0) + 80 * 256) / 256 * 255 / 80;
    let mult: Q15 = DB_LINEAR_AMPLITUDE_TABLE[index as usize].lossy_into();
    mult
}

static VOLUME_STATE: StaticCell<VolumeState> = StaticCell::new();

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
        let volume_mult = volume_state.volume_mult.load(Ordering::Relaxed);

        // Copy and multiply in one operation c:
        CmsisNativeOperations::multiply_q15(
            bytemuck::cast_slice::<i16, Q15>(samples),
            &[Q15::from_bits(volume_mult); USB_MAX_SAMPLE_COUNT],
            bytemuck::cast_mut::<[u8; USB_MAX_PACKET_SIZE], [Q15; USB_MAX_SAMPLE_COUNT]>(
                &mut usb_data,
            ),
        );

        receiver.receive_done();

        stream.write_packet(&usb_data).await?;

        packet_count += 1;
        if packet_count % 1000 == 0 {
            info!(
                "USB Audio: Streamed {} packets at volume {}",
                packet_count, volume_mult
            );
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
                    volume_state.volume_mult.store(0, Ordering::Relaxed);
                    info!("USB Audio: Set volume_mult to 0");
                }
                Volume::DeciBel(vol_8q8) => {
                    let db_int = vol_8q8 / 256;
                    info!(
                        "USB Audio: Channel {} gain: {} dB (raw: {})",
                        channel, db_int, vol_8q8
                    );
                    let mult = volume_8q8_to_mult(vol_8q8).to_bits();
                    volume_state.volume_mult.store(mult, Ordering::Relaxed);
                    info!("USB Audio: Set volume_mult to {}", mult);
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

    let volume_state = VOLUME_STATE.init(VolumeState {
        volume_mult: AtomicI16::new(i16::MAX),
    });
    info!("USB Audio: Set volume_mult to {}", i16::MAX);

    let sample_blocks = super::SAMPLE_BLOCKS.init([[0; super::USB_MAX_SAMPLE_COUNT]; 2]);
    let channel = super::AUDIO_CHANNEL.init(zerocopy_channel::Channel::new(sample_blocks));
    let (sender, receiver) = channel.split();

    let control_task = usb_control_task(audio_hardware.control_monitor, volume_state);
    let streaming_task = usb_streaming_task(audio_hardware.stream, receiver, volume_state);

    info!("USB Audio: Tasks created successfully");

    (control_task, streaming_task, sender)
}
