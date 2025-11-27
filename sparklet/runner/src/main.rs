#![no_std]
#![no_main]

mod hardware;
mod midi_task;
mod synth_engine_task;

#[cfg(feature = "usb")]
mod usb;
#[cfg(feature = "audio-usb")]
mod audio_task;

use defmt::info;
use embassy_executor::Executor;
use static_cell::StaticCell;

use defmt_rtt as _;
use embassy_stm32 as _;
use panic_probe as _;

#[cfg(feature = "usb")]
#[embassy_executor::task]
async fn usb_device_task(
    mut usb_device: embassy_usb::UsbDevice<
        'static,
        embassy_stm32::usb::Driver<'static, embassy_stm32::peripherals::USB_OTG_HS>,
    >,
) {
    usb_device.run().await;
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Setting up hardware");
    let hardware = hardware::Hardware::get();

    info!("Setting up executor");
    let executor = EXECUTOR.init(embassy_executor::Executor::new());

    #[cfg(any(feature = "midi-din", feature = "midi-usb"))]
    let midi_task = {
        info!("Creating MIDI task");
        midi_task::create_midi_task(hardware.midi_hardware)
    };

    #[cfg(feature = "usb")]
    let usb_device = {
        info!("Building USB device");
        hardware.usb_builder.build()
    };

    #[cfg(feature = "audio-usb")]
    let (audio_control_task, audio_streaming_task, audio_sender) = {
        info!("Creating audio tasks");
        audio_task::create_audio_tasks(hardware.audio_hardware)
    };

    info!("Creating synth engine task");
    #[cfg(feature = "audio-usb")]
    let synth_engine_task = synth_engine_task::create_task(audio_sender);

    #[cfg(not(feature = "audio-usb"))]
    let synth_engine_task = synth_engine_task::create_task();

    info!("Setting up tasks in executors...");
    executor.run(|spawner| {
        #[cfg(any(feature = "midi-din", feature = "midi-usb"))]
        spawner.spawn(midi_task).unwrap();

        spawner.spawn(synth_engine_task).unwrap();

        #[cfg(feature = "usb")]
        {
            info!("Spawning USB device task");
            spawner.spawn(usb_device_task(usb_device)).unwrap();
        }

        #[cfg(feature = "audio-usb")]
        {
            info!("Spawning audio tasks");
            spawner.spawn(audio_control_task).unwrap();
            spawner.spawn(audio_streaming_task).unwrap();
        }
    });
}
