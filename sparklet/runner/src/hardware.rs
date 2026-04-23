use defmt::info;
#[cfg(feature = "configurable")]
use embassy_stm32::gpio::{Input, Pull};
#[cfg(feature = "configurable")]
use embassy_stm32::peripherals::{TIM2, TIM3, TIM4};
#[cfg(feature = "configurable")]
use embassy_stm32::timer::qei::{Qei, QeiPin};
#[cfg(feature = "configurable")]
use static_cell::StaticCell;

#[cfg(feature = "configurable")]
use crate::hardware_abstractions::{ActiveLow, InputWithPolarity};
#[cfg(feature = "configurable")]
use crate::hardware_abstractions::{Button, QeiExt};

#[cfg(feature = "configurable")]
pub struct InputHardware {
    pub button_next_page: &'static dyn Button,
    pub button_prev_page: &'static dyn Button,
    pub encoder0: &'static dyn QeiExt,
    pub encoder1: &'static dyn QeiExt,
    pub encoder2: &'static dyn QeiExt,
}

#[cfg(feature = "configurable")]
pub static BUTTON_NEXT_PAGE: StaticCell<InputWithPolarity<ActiveLow>> = StaticCell::new();
#[cfg(feature = "configurable")]
pub static BUTTON_PREV_PAGE: StaticCell<InputWithPolarity<ActiveLow>> = StaticCell::new();
#[cfg(feature = "configurable")]
pub static ENCODER0_QEI: StaticCell<Qei<TIM2>> = StaticCell::new();
#[cfg(feature = "configurable")]
pub static ENCODER1_QEI: StaticCell<Qei<TIM3>> = StaticCell::new();
#[cfg(feature = "configurable")]
pub static ENCODER2_QEI: StaticCell<Qei<TIM4>> = StaticCell::new();

#[cfg(feature = "configurable")]
impl InputHardware {
    pub fn new(
        button_next_page: &'static mut InputWithPolarity<'static, ActiveLow>,
        button_prev_page: &'static mut InputWithPolarity<'static, ActiveLow>,
        encoder0: &'static mut Qei<'static, TIM2>,
        encoder1: &'static mut Qei<'static, TIM3>,
        encoder2: &'static mut Qei<'static, TIM4>,
    ) -> Self {
        Self {
            button_next_page,
            button_prev_page,
            encoder0,
            encoder1,
            encoder2,
        }
    }
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
    #[cfg(feature = "configurable")]
    pub input_hardware: InputHardware,
}

impl Hardware {
    pub fn get() -> Hardware {
        info!("Initializing");

        let mut config = embassy_stm32::Config::default();
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

        #[cfg(feature = "configurable")]
        let input_hardware = {
            InputHardware::new(
                // A0, left, left, first from top of split
                BUTTON_NEXT_PAGE.init(InputWithPolarity::<ActiveLow>::new(Input::new(
                    peripherals.PA3,
                    Pull::Up,
                ))),
                // Internal
                BUTTON_PREV_PAGE.init(InputWithPolarity::<ActiveLow>::new(Input::new(
                    peripherals.PC13,
                    Pull::None,
                ))),
                ENCODER0_QEI.init(Qei::new(
                    peripherals.TIM2,
                    // D20, right, left, fifth from top
                    QeiPin::new(peripherals.PA15),
                    // D23, right, left, eight from top
                    QeiPin::new(peripherals.PB3),
                )),
                ENCODER1_QEI.init(Qei::new(
                    peripherals.TIM3,
                    // D12, right, right, sixth from top
                    QeiPin::new(peripherals.PA6),
                    // D23, D11, right, right, seventh from top
                    QeiPin::new(peripherals.PB5),
                )),
                ENCODER2_QEI.init(Qei::new(
                    peripherals.TIM4,
                    // D1 right, right, seventh from top of split
                    QeiPin::new(peripherals.PB6),
                    // D0 right, right, eigth from top of split
                    QeiPin::new(peripherals.PB7),
                )),
            )
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
            #[cfg(feature = "configurable")]
            input_hardware,
        }
    }
}
