use defmt::info;
use embassy_stm32::Config;

pub struct Hardware {
    #[cfg(feature = "midi-din")]
    pub midi_hardware: crate::midi_task::midi_din::hardware::MidiDinHardware<'static>,
    #[cfg(feature = "midi-usb")]
    pub midi_hardware: crate::midi_task::midi_usb::hardware::MidiUsbHardware<'static>,
    #[cfg(feature = "usb")]
    pub usb_builder: embassy_usb::Builder<
        'static,
        embassy_stm32::usb::Driver<'static, embassy_stm32::peripherals::USB_OTG_HS>,
    >,
    #[cfg(feature = "audio-usb")]
    pub audio_hardware: crate::audio_task::audio_usb::hardware::AudioUsbHardware<'static>,
}

impl Hardware {
    pub fn get() -> Hardware {
        info!("Initializing");

        let mut config = Config::default();
        #[cfg(feature = "usb")]
        {
            info!("USB config being added...");
            use embassy_stm32::rcc::*;
            // Configure clocks for STM32H7
            config.rcc.hsi = Some(HSIPrescaler::DIV1);
            config.rcc.csi = true;
            config.rcc.hsi48 = Some(Hsi48Config {
                sync_from_usb: true,
            }); // needed for USB
            config.rcc.pll1 = Some(Pll {
                source: PllSource::HSI,
                prediv: PllPreDiv::DIV4,
                mul: PllMul::MUL50,
                divp: Some(PllDiv::DIV2), // 400 MHz
                divq: None,
                divr: None,
            });
            config.rcc.sys = Sysclk::PLL1_P; // 400 MHz
            config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 MHz
            config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 MHz
            config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 MHz
            config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 MHz
            config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 MHz
            config.rcc.voltage_scale = VoltageScale::Scale1;
            config.rcc.mux.usbsel = mux::Usbsel::HSI48;
            info!("USB config added.")
        }

        let peripherals = embassy_stm32::init(config);

        #[cfg(feature = "midi-din")]
        let midi_hardware = crate::get_midi_din_hardware!(peripherals);

        #[cfg(feature = "usb")]
        let usb_hardware = crate::get_usb_hardware!(peripherals);

        #[cfg(feature = "usb")]
        let mut usb_config = embassy_usb::Config::new(0xc0de, 0xcafe);
        #[cfg(feature = "usb")]
        {
            usb_config.manufacturer = Some("Sparklet");
            usb_config.product = Some("Sparklet Synth");
            usb_config.serial_number = Some("12345678");
        }

        #[cfg(feature = "usb")]
        let mut usb_builder = embassy_usb::Builder::new(
            usb_hardware.driver,
            usb_config,
            usb_hardware.config_descriptor,
            usb_hardware.bos_descriptor,
            &mut [],
            usb_hardware.control_buf,
        );

        #[cfg(feature = "midi-usb")]
        let midi_hardware = crate::get_midi_usb_hardware!(&mut usb_builder);

        #[cfg(feature = "audio-usb")]
        let audio_hardware = crate::get_audio_usb_hardware!(&mut usb_builder);

        Hardware {
            #[cfg(feature = "midi-din")]
            midi_hardware,
            #[cfg(feature = "midi-usb")]
            midi_hardware,
            #[cfg(feature = "usb")]
            usb_builder,
            #[cfg(feature = "audio-usb")]
            audio_hardware,
        }
    }
}
