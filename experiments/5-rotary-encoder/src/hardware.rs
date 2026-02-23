use defmt::info;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Pull;

pub struct Hardware<'a> {
    pub encoder_a: ExtiInput<'a>,
    pub encoder_b: ExtiInput<'a>,
}

impl<'a> Hardware<'a> {
    pub fn get() -> Hardware<'a> {
        info!("Initializing");
        let peripherals = embassy_stm32::init(Default::default());

        let encoder_a = ExtiInput::new(peripherals.PA3, peripherals.EXTI3, Pull::Up);
        let encoder_b = ExtiInput::new(peripherals.PC0, peripherals.EXTI0, Pull::Up);

        Hardware {
            encoder_a,
            encoder_b,
        }
    }
}
