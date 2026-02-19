#![cfg_attr(not(test), no_std)]

use cmsis_interface::CmsisOperations;
use fixed::types::I34F30;

pub struct CmsisRustOperations;

impl CmsisOperations for CmsisRustOperations {
    fn abs_in_place_q15(values: &mut [cmsis_interface::Q15]) {
        for value in values.iter_mut() {
            *value = value.abs();
        }
    }

    fn abs_q15(src: &[cmsis_interface::Q15], dst: &mut [cmsis_interface::Q15]) {
        if src.len() != dst.len() {
            panic!("src.len() != dst.len()");
        }

        for (dst, src) in dst.iter_mut().zip(src.iter()) {
            *dst = src.abs();
        }
    }

    fn add_q15(
        src1: &[cmsis_interface::Q15],
        src2: &[cmsis_interface::Q15],
        dst: &mut [cmsis_interface::Q15],
    ) {
        if (src1.len() != src2.len()) || (src1.len() != dst.len()) {
            panic!("src1.len() != src2.len() || src1.len() != dst.len()");
        }

        for (dst, (src1, src2)) in dst.iter_mut().zip(src1.iter().zip(src2.iter())) {
            *dst = src1.saturating_add(*src2);
        }
    }

    fn multiply_q15(
        src1: &[cmsis_interface::Q15],
        src2: &[cmsis_interface::Q15],
        dst: &mut [cmsis_interface::Q15],
    ) {
        if (src1.len() != src2.len()) || (src1.len() != dst.len()) {
            panic!("src1.len() != src2.len() || src1.len() != dst.len()");
        }

        for (dst, (src1, src2)) in dst.iter_mut().zip(src1.iter().zip(src2.iter())) {
            *dst = src1.saturating_mul(*src2);
        }
    }

    fn negate_in_place_q15(values: &mut [cmsis_interface::Q15]) {
        for value in values.iter_mut() {
            *value = value.saturating_neg();
        }
    }

    fn negate_q15(src: &[cmsis_interface::Q15], dst: &mut [cmsis_interface::Q15]) {
        if src.len() != dst.len() {
            panic!("src.len() != dst.len()");
        }

        for (dst, src) in dst.iter_mut().zip(src.iter()) {
            *dst = src.saturating_neg();
        }
    }

    fn shift_q15(src: &[cmsis_interface::Q15], shift_bits: i8, dst: &mut [cmsis_interface::Q15]) {
        if src.len() != dst.len() {
            panic!("src.len() != dst.len()");
        }

        for (dst, src) in dst.iter_mut().zip(src.iter()) {
            *dst = if shift_bits >= 0 {
                // Left shift
                *src << shift_bits
            } else {
                // Right shift
                *src >> (-shift_bits)
            };
        }
    }

    fn shift_in_place_q15(values: &mut [cmsis_interface::Q15], shift_bits: i8) {
        for value in values.iter_mut() {
            *value = if shift_bits >= 0 {
                // Left shift
                *value << shift_bits
            } else {
                // Right shift
                *value >> (-shift_bits)
            };
        }
    }

    fn biquad_cascade_df1_q15<const NUM_STAGES: usize, const STATE_SIZE: usize>(
        state: &mut cmsis_interface::BiquadCascadeDf1StateQ15<NUM_STAGES, STATE_SIZE>,
        coeffs: &[[cmsis_interface::Q15; 6]; NUM_STAGES],
        post_shift: i8,
        input: &[cmsis_interface::Q15],
        output: &mut [cmsis_interface::Q15],
    ) {
        use cmsis_interface::Q15;

        if input.len() != output.len() {
            panic!("input.len() != output.len()");
        }

        for (input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
            let mut sample = *input_sample;
            let state_slice = state.as_mut_slice();

            for (stage_idx, stage_coeffs) in coeffs.iter().enumerate() {
                // CMSIS format: [b0, 0, b1, b2, a1, a2]
                let b0 = stage_coeffs[0];
                let b1 = stage_coeffs[2];
                let b2 = stage_coeffs[3];
                let a1 = stage_coeffs[4];
                let a2 = stage_coeffs[5];

                let state_offset = stage_idx * 4;
                let x_n1 = state_slice[state_offset];
                let x_n2 = state_slice[state_offset + 1];
                let y_n1 = state_slice[state_offset + 2];
                let y_n2 = state_slice[state_offset + 3];

                let acc = I34F30::from(b0.wide_mul(sample))
                    + I34F30::from(b1.wide_mul(x_n1))
                    + I34F30::from(b2.wide_mul(x_n2))
                    + I34F30::from(a1.wide_mul(y_n1))
                    + I34F30::from(a2.wide_mul(y_n2));

                let result = Q15::saturating_from_num(acc.unbounded_shl(post_shift as u32));

                state_slice[state_offset + 1] = x_n1; // x[n-2] = x[n-1]
                state_slice[state_offset] = sample; // x[n-1] = x[n]
                state_slice[state_offset + 3] = y_n1; // y[n-2] = y[n-1]
                state_slice[state_offset + 2] = result; // y[n-1] = y[n]

                sample = result;
            }

            *output_sample = sample;
        }
    }
}

cmsis_interface::declare_tests!(crate::CmsisRustOperations,,);
