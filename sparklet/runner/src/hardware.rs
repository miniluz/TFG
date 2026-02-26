use defmt::info;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::Config;

pub struct InputHardware<'a> {
    pub button_page_down: ExtiInput<'a>,
    pub button_page_up: ExtiInput<'a>,
    pub encoder0_exti: ExtiInput<'a>,
    pub encoder0_input: Input<'a>,
    pub encoder1_exti: ExtiInput<'a>,
    pub encoder1_input: Input<'a>,
    pub encoder2_exti: ExtiInput<'a>,
    pub encoder2_input: Input<'a>,
}

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
    pub input_hardware: InputHardware<'static>,
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

        #[cfg(feature = "usb")]
        let mut usb_builder = {
            let usb_hardware = crate::get_usb_hardware!(peripherals);

            let mut usb_config = embassy_usb::Config::new(0xc0de, 0xcafe);

            usb_config.manufacturer = Some("miniluz");
            usb_config.product = Some("Sparklet Synth");
            usb_config.serial_number = Some("12345678");

            embassy_usb::Builder::new(
                usb_hardware.driver,
                usb_config,
                usb_hardware.config_descriptor,
                usb_hardware.bos_descriptor,
                &mut [],
                usb_hardware.control_buf,
            )
        };

        #[cfg(feature = "midi-din")]
        let midi_hardware = crate::get_midi_din_hardware!(peripherals);

        #[cfg(feature = "midi-usb")]
        let midi_hardware = crate::get_midi_usb_hardware!(&mut usb_builder);

        #[cfg(feature = "audio-usb")]
        let audio_hardware = crate::get_audio_usb_hardware!(&mut usb_builder);

        let input_hardware = InputHardware {
            button_page_down: ExtiInput::new(peripherals.PA3, peripherals.EXTI3, Pull::Up),
            button_page_up: ExtiInput::new(peripherals.PC13, peripherals.EXTI13, Pull::None),
            encoder0_exti: ExtiInput::new(peripherals.PB1, peripherals.EXTI1, Pull::Up),
            encoder0_input: Input::new(peripherals.PB2, Pull::Up),
            encoder1_exti: ExtiInput::new(peripherals.PC2, peripherals.EXTI2, Pull::Up),
            encoder1_input: Input::new(peripherals.PE9, Pull::Up),
            encoder2_exti: ExtiInput::new(peripherals.PF10, peripherals.EXTI10, Pull::Up),
            encoder2_input: Input::new(peripherals.PF2, Pull::Up),
        };

        Hardware {
            #[cfg(feature = "midi-din")]
            midi_hardware,
            #[cfg(feature = "midi-usb")]
            midi_hardware,
            #[cfg(feature = "usb")]
            usb_builder,
            #[cfg(feature = "audio-usb")]
            audio_hardware,
            input_hardware,
        }
    }
}
