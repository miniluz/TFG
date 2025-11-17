#![cfg_attr(not(test), no_std)]

use cmsis_interface::CmsisOperations;

pub struct CmsisRustOperations;

impl CmsisOperations for CmsisRustOperations {
    fn add(left: u64, right: u64) -> u64 {
        left + right
    }

    fn multiply_f32(src1: &[f32], src2: &[f32], dst: &mut [f32]) {
        {
            if (src1.len() != src2.len()) || (src1.len() != dst.len()) {
                panic!("src1.len() != src2.len() || src1.len() != dst.len()");
            }

            for (dst, (src1, src2)) in dst.iter_mut().zip(src1.iter().zip(src2.iter())) {
                *dst = *src1 * *src2;
            }
        }
    }
}

cmsis_interface::declare_tests!(crate::CmsisRustOperations,,);
