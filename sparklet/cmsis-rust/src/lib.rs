#![cfg_attr(not(test), no_std)]

use cmsis_interface::CmsisOperations;

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
            // TODO: Check if they saturate
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
            // Multiplication should never saturate...
            *dst = src1 * src2;
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

    fn shift_q15(
        src: &[cmsis_interface::Q15],
        shift_bits: i8,
        dst: &mut [cmsis_interface::Q15],
    ) {
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
}

cmsis_interface::declare_tests!(crate::CmsisRustOperations,,);
