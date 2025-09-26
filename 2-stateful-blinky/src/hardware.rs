use defmt::info;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};

pub struct Hardware<'a> {
    pub left_led: Output<'a>,
    pub middle_led: Output<'a>,
    pub right_led: Output<'a>,
    pub button: Button<'a>,
}

pub enum Polarity {
    ActiveLow,
    ActiveHigh,
}

pub struct Button<'a> {
    button: ExtiInput<'a>,
    polarity: Polarity,
}

impl<'a> Button<'a> {
    pub fn new(button: ExtiInput<'a>, polarity: Polarity) -> Button<'a> {
        Button { button, polarity }
    }

    pub async fn wait_for_pressed(&mut self) {
        match self.polarity {
            Polarity::ActiveHigh => self.button.wait_for_high().await,
            Polarity::ActiveLow => self.button.wait_for_low().await,
        }
    }

    pub async fn wait_for_released(&mut self) {
        match self.polarity {
            Polarity::ActiveHigh => self.button.wait_for_low().await,
            Polarity::ActiveLow => self.button.wait_for_high().await,
        }
    }
}

impl<'a> Hardware<'a> {
    pub fn get() -> Hardware<'a> {
        info!("Initializing");
        let peripherals = embassy_stm32::init(Default::default());

        let left_led = Output::new(peripherals.PB0, Level::Low, Speed::Low);
        let middle_led = Output::new(peripherals.PE1, Level::Low, Speed::Low);
        let right_led = Output::new(peripherals.PB14, Level::Low, Speed::Low);

        let button = ExtiInput::new(
            peripherals.PC13,
            peripherals.EXTI13,
            Pull::None, // No need for internal pulls, as the board has an external pull down
        );
        let button = Button::new(button, Polarity::ActiveHigh);

        Hardware {
            left_led,
            middle_led,
            right_led,
            button,
        }
    }
}
