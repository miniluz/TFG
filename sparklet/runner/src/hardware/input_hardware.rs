use embassy_stm32::{
    Peri,
    gpio::{Input, Pull},
    timer::qei::{Qei, QeiPin},
};
use static_cell::StaticCell;

use crate::hardware::abstractions::{ActiveLow, Button, InputWithPolarity, QeiExt};
use embassy_stm32::peripherals::{PA3, PA6, PA15, PB3, PB5, PB6, PB7, PC13, TIM2, TIM3, TIM4};

pub struct InputHardware {
    pub button_next_page: &'static dyn Button,
    pub button_prev_page: &'static dyn Button,
    pub encoder0: &'static dyn QeiExt,
    pub encoder1: &'static dyn QeiExt,
    pub encoder2: &'static dyn QeiExt,
}

pub static BUTTON_NEXT_PAGE: StaticCell<InputWithPolarity<ActiveLow>> = StaticCell::new();
pub static BUTTON_PREV_PAGE: StaticCell<InputWithPolarity<ActiveLow>> = StaticCell::new();
pub static ENCODER0_QEI: StaticCell<Qei<TIM2>> = StaticCell::new();
pub static ENCODER1_QEI: StaticCell<Qei<TIM3>> = StaticCell::new();
pub static ENCODER2_QEI: StaticCell<Qei<TIM4>> = StaticCell::new();

impl InputHardware {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        // A0, left, left, first from top of split
        pin_next_page: Peri<'static, PA3>,
        // Internal
        pin_prev_page: Peri<'static, PC13>,
        encoder0_tim: Peri<'static, TIM2>,
        // D20, right, left, fifth from top
        encoder0_ch1: Peri<'static, PA15>,
        // D23, right, left, eight from top
        encoder0_ch2: Peri<'static, PB3>,
        encoder1_tim: Peri<'static, TIM3>,
        // D12, right, right, sixth from top
        encoder1_ch1: Peri<'static, PA6>,
        // D23, D11, right, right, seventh from top
        encoder1_ch2: Peri<'static, PB5>,
        encoder2_tim: Peri<'static, TIM4>,
        // D1 right, right, seventh from top of split
        encoder2_ch1: Peri<'static, PB6>,
        // D0 right, right, eigth from top of split
        encoder2_ch2: Peri<'static, PB7>,
    ) -> Self {
        Self {
            button_next_page: BUTTON_NEXT_PAGE.init(InputWithPolarity::<ActiveLow>::new(
                Input::new(pin_next_page, Pull::Up),
            )),
            button_prev_page: BUTTON_PREV_PAGE.init(InputWithPolarity::<ActiveLow>::new(
                Input::new(pin_prev_page, Pull::None),
            )),
            encoder0: ENCODER0_QEI.init(Qei::new(
                encoder0_tim,
                QeiPin::new(encoder0_ch1),
                QeiPin::new(encoder0_ch2),
            )),
            encoder1: ENCODER1_QEI.init(Qei::new(
                encoder1_tim,
                QeiPin::new(encoder1_ch1),
                QeiPin::new(encoder1_ch2),
            )),
            encoder2: ENCODER2_QEI.init(Qei::new(
                encoder2_tim,
                QeiPin::new(encoder2_ch1),
                QeiPin::new(encoder2_ch2),
            )),
        }
    }
}
