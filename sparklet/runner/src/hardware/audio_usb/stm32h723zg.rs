use embassy_stm32::peripherals;
use embassy_stm32::usb;
use embassy_usb::class::uac1;
use embassy_usb::class::uac1::microphone;
use static_cell::StaticCell;

pub struct AudioUsbHardware<'d> {
    pub stream: microphone::Stream<'d, usb::Driver<'d, peripherals::USB_OTG_HS>>,
    pub control_monitor: microphone::ControlMonitor<'d>,
}

// Mono input (microphone simulation)
pub const INPUT_CHANNEL_COUNT: usize = 1;

// Fixed sample rate of 48 kHz
pub const SAMPLE_RATE_HZ: u32 = 48_000;

// Use 16 bit samples for microphone input
pub const SAMPLE_WIDTH: uac1::SampleWidth = uac1::SampleWidth::Width2Byte;
pub const SAMPLE_SIZE: usize = SAMPLE_WIDTH as usize;
pub const SAMPLE_SIZE_PER_S: usize = (SAMPLE_RATE_HZ as usize) * INPUT_CHANNEL_COUNT * SAMPLE_SIZE;

// Size of audio samples per 1 ms - for the full-speed USB frame period of 1 ms
pub const USB_FRAME_SIZE: usize = SAMPLE_SIZE_PER_S.div_ceil(1000);

// Select mono audio channel (left front)
pub const AUDIO_CHANNELS: [uac1::Channel; INPUT_CHANNEL_COUNT] = [uac1::Channel::LeftFront];

// USB packet size for microphone (synchronous mode, no margin needed)
pub const USB_MAX_PACKET_SIZE: usize = USB_FRAME_SIZE;
pub const USB_MAX_SAMPLE_COUNT: usize = USB_MAX_PACKET_SIZE / SAMPLE_SIZE;

pub static STATE: StaticCell<microphone::State> = StaticCell::new();

#[macro_export]
macro_rules! get_audio_usb_hardware {
    ($builder:expr) => {{
        use embassy_usb::class::uac1::microphone::{Microphone, State};
        use $crate::hardware::audio_usb::{
            AUDIO_CHANNELS, AudioUsbHardware, SAMPLE_RATE_HZ, SAMPLE_WIDTH, STATE,
            USB_MAX_PACKET_SIZE,
        };

        let state = STATE.init(State::new());

        // Create the UAC1 Microphone class components (synchronous mode)
        let (stream, control_monitor) = Microphone::new(
            $builder,
            state,
            USB_MAX_PACKET_SIZE as u16,
            SAMPLE_WIDTH,
            &[SAMPLE_RATE_HZ],
            &AUDIO_CHANNELS,
        );

        AudioUsbHardware {
            stream,
            control_monitor,
        }
    }};
}
