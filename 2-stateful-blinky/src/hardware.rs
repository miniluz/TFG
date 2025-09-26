use defmt::info;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};

pub struct Hardware<'a> {
    pub left_led: Output<'a>,
    pub middle_led: Output<'a>,
    pub right_led: Output<'a>,
    pub button: ExtiInput<'a>,
}

impl<'a> Hardware<'a> {
    pub fn get() -> Hardware<'a> {
        info!("Initializing");
        let peripherals = embassy_stm32::init(Default::default());

        let left_led = Output::new(peripherals.PB0, Level::Low, Speed::Low);
        let middle_led = Output::new(peripherals.PE1, Level::Low, Speed::Low);
        let right_led = Output::new(peripherals.PB14, Level::Low, Speed::Low);

        let button = ExtiInput::new(peripherals.PC13, peripherals.EXTI13, Pull::Down);

        Hardware {
            left_led,
            middle_led,
            right_led,
            button,
        }
    }
}
