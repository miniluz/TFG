#![no_std]
#![no_main]

use cmsis_interface::{CmsisOperations, Q15};

pub struct CmsisNativeOperations;

impl CmsisOperations for CmsisNativeOperations {
    fn abs_in_place_q15(values: &mut [Q15]) {
        cmsis_dsp::basic::abs_in_place_q15(values);
    }

    fn abs_q15(src: &[Q15], dst: &mut [Q15]) {
        cmsis_dsp::basic::abs_q15(src, dst);
    }

    fn add_q15(src1: &[Q15], src2: &[Q15], dst: &mut [Q15]) {
        cmsis_dsp::basic::add_q15(src1, src2, dst);
    }

    fn multiply_q15(src1: &[Q15], src2: &[Q15], dst: &mut [Q15]) {
        cmsis_dsp::basic::multiply_q15(src1, src2, dst);
    }

    fn negate_in_place_q15(values: &mut [Q15]) {
        cmsis_dsp::basic::negate_in_place_q15(values);
    }

    fn negate_q15(src: &[Q15], dst: &mut [Q15]) {
        cmsis_dsp::basic::negate_q15(src, dst);
    }

    fn shift_q15(src: &[Q15], shift_bits: i8, dst: &mut [Q15]) {
        cmsis_dsp::basic::shift_q15(src, shift_bits, dst);
    }

    fn shift_in_place_q15(values: &mut [Q15], shift_bits: i8) {
        cmsis_dsp::basic::shift_in_place_q15(values, shift_bits);
    }
}

#[cfg(test)]
#[embedded_test::setup]
fn setup() {
    rtt_target::rtt_init_defmt!();
}

#[cfg(test)]
cmsis_interface::declare_tests! {
    crate::CmsisNativeOperations,
    #[embedded_test::tests],
    use embassy_stm32 as _;
}
