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

    fn biquad_cascade_df1_q15<const NUM_STAGES: usize, const STATE_SIZE: usize>(
        state: &mut cmsis_interface::BiquadCascadeDf1StateQ15<NUM_STAGES, STATE_SIZE>,
        coeffs: &[[Q15; 6]; NUM_STAGES],
        post_shift: i8,
        input: &[Q15],
        output: &mut [Q15],
    ) {
        let coeffs_flat: &[Q15] = bytemuck::cast_slice(coeffs);

        let mut inst = cmsis_dsp::filter::BiquadCascadeDf1InstQ15::new(
            NUM_STAGES,
            state.as_mut_slice(),
            coeffs_flat,
            post_shift,
        );

        cmsis_dsp::filter::biquad_cascade_df1_q15(&mut inst, input, output);
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
