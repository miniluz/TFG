use core::marker::PhantomData;
use embassy_stm32::{
    gpio::Input,
    timer::{GeneralInstance4Channel, qei::Qei},
};

pub struct InputWithPolarity<'a, T> {
    input: Input<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> InputWithPolarity<'a, T> {
    pub fn new(input: Input<'a>) -> Self {
        Self {
            input,
            phantom: PhantomData,
        }
    }
}

pub struct ActiveHigh;
pub struct ActiveLow;

pub trait Button {
    fn is_pressed(&self) -> bool;
}

impl<'a> Button for InputWithPolarity<'a, ActiveHigh> {
    fn is_pressed(&self) -> bool {
        self.input.is_high()
    }
}

impl<'a> Button for InputWithPolarity<'a, ActiveLow> {
    fn is_pressed(&self) -> bool {
        self.input.is_low()
    }
}

pub trait QeiExt {
    fn count(&self) -> u16;
}

impl<'a, T: GeneralInstance4Channel> QeiExt for Qei<'a, T> {
    fn count(&self) -> u16 {
        self.count()
    }
}
