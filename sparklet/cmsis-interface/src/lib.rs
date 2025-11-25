#![no_std]

pub use fixed::types::I1F15 as Q15;

pub trait CmsisOperations {
    fn abs_in_place_q15(values: &mut [Q15]);
    fn abs_q15(src: &[Q15], dst: &mut [Q15]);
    fn add_q15(src1: &[Q15], src2: &[Q15], dst: &mut [Q15]);
    fn multiply_q15(src1: &[Q15], src2: &[Q15], dst: &mut [Q15]);
    fn negate_in_place_q15(values: &mut [Q15]);
    fn negate_q15(src: &[Q15], dst: &mut [Q15]);
    fn shift_q15(src: &[Q15], shift_bits: i8, dst: &mut [Q15]);
    fn shift_in_place_q15(values: &mut [Q15], shift_bits: i8);
}

#[macro_export]
macro_rules! declare_tests {
    {$T:ty, $(#[$meta:meta]),*, $($prelude:tt)*} => {
        #[cfg(test)]
        $(#[$meta])*
        mod tests {

            $($prelude)*

            use cmsis_interface::{CmsisOperations, Q15};

            #[test]
            pub fn test_multiply_q15() {
                let src1 = [Q15::from_num(0.5), Q15::from_num(0.5), Q15::from_num(0.25)];
                let src2 = [Q15::from_num(0.5), Q15::from_num(0.25), Q15::from_num(0.5)];
                let mut dst = [Q15::from_num(0.0); 3];
                <$T>::multiply_q15(&src1, &src2, &mut dst);
                assert_eq!(dst, [Q15::from_num(0.25), Q15::from_num(0.125), Q15::from_num(0.125)]);
            }

            #[test]
            pub fn test_add_q15() {
                let src1 = [Q15::from_num(0.9), Q15::from_num(0.25), Q15::from_num(-0.9)];
                let src2 = [Q15::from_num(0.9), Q15::from_num(0.5), Q15::from_num(-0.9)];
                let mut dst = [Q15::from_num(0.0); 3];
                <$T>::add_q15(&src1, &src2, &mut dst);
                // 0.9 + 0.9 = 1.8, should saturate to max (~0.99997)
                // -0.9 + -0.9 = -1.8, should saturate to min (-1.0)
                assert_eq!(dst[0], Q15::MAX);
                assert_eq!(dst[1], Q15::from_num(0.75));
                assert_eq!(dst[2], Q15::MIN);
            }

            #[test]
            pub fn test_abs_q15() {
                let src = [Q15::from_num(-0.5), Q15::from_num(0.25), Q15::from_num(-0.75)];
                let mut dst = [Q15::from_num(0.0); 3];
                <$T>::abs_q15(&src, &mut dst);
                assert_eq!(dst, [Q15::from_num(0.5), Q15::from_num(0.25), Q15::from_num(0.75)]);
            }

            #[test]
            pub fn test_abs_in_place_q15() {
                let mut values = [Q15::from_num(-0.5), Q15::from_num(0.25), Q15::from_num(-0.75)];
                <$T>::abs_in_place_q15(&mut values);
                assert_eq!(values, [Q15::from_num(0.5), Q15::from_num(0.25), Q15::from_num(0.75)]);
            }

            #[test]
            pub fn test_negate_q15() {
                let src = [Q15::from_num(0.5), Q15::from_num(-0.25), Q15::from_num(0.75)];
                let mut dst = [Q15::from_num(0.0); 3];
                <$T>::negate_q15(&src, &mut dst);
                assert_eq!(dst, [Q15::from_num(-0.5), Q15::from_num(0.25), Q15::from_num(-0.75)]);
            }

            #[test]
            pub fn test_negate_in_place_q15() {
                let mut values = [Q15::from_num(0.5), Q15::from_num(-0.25), Q15::from_num(0.75)];
                <$T>::negate_in_place_q15(&mut values);
                assert_eq!(values, [Q15::from_num(-0.5), Q15::from_num(0.25), Q15::from_num(-0.75)]);
            }

            #[test]
            pub fn test_shift_q15_left() {
                let src = [Q15::from_num(0.25), Q15::from_num(0.125), Q15::from_num(0.0625)];
                let mut dst = [Q15::from_num(0.0); 3];
                <$T>::shift_q15(&src, 1, &mut dst);
                assert_eq!(dst, [Q15::from_num(0.5), Q15::from_num(0.25), Q15::from_num(0.125)]);
            }

            #[test]
            pub fn test_shift_q15_right() {
                let src = [Q15::from_num(0.5), Q15::from_num(0.25), Q15::from_num(0.125)];
                let mut dst = [Q15::from_num(0.0); 3];
                <$T>::shift_q15(&src, -1, &mut dst);
                assert_eq!(dst, [Q15::from_num(0.25), Q15::from_num(0.125), Q15::from_num(0.0625)]);
            }

            #[test]
            pub fn test_shift_in_place_q15_left() {
                let mut values = [Q15::from_num(0.25), Q15::from_num(0.125), Q15::from_num(0.0625)];
                <$T>::shift_in_place_q15(&mut values, 1);
                assert_eq!(values, [Q15::from_num(0.5), Q15::from_num(0.25), Q15::from_num(0.125)]);
            }

            #[test]
            pub fn test_shift_in_place_q15_right() {
                let mut values = [Q15::from_num(0.5), Q15::from_num(0.25), Q15::from_num(0.125)];
                <$T>::shift_in_place_q15(&mut values, -1);
                assert_eq!(values, [Q15::from_num(0.25), Q15::from_num(0.125), Q15::from_num(0.0625)]);
            }

            #[test]
            pub fn test_shift_q15_zero() {
                let src = [Q15::from_num(0.5), Q15::from_num(0.25), Q15::from_num(-0.5)];
                let mut dst = [Q15::from_num(0.0); 3];
                <$T>::shift_q15(&src, 0, &mut dst);
                assert_eq!(dst, [Q15::from_num(0.5), Q15::from_num(0.25), Q15::from_num(-0.5)]);
            }

            #[test]
            pub fn test_shift_q15_negative_values() {
                let src = [Q15::from_num(-0.5), Q15::from_num(-0.25), Q15::from_num(-0.125)];
                let mut dst = [Q15::from_num(0.0); 3];
                <$T>::shift_q15(&src, -1, &mut dst);
                assert_eq!(dst, [Q15::from_num(-0.25), Q15::from_num(-0.125), Q15::from_num(-0.0625)]);
            }

            #[test]
            pub fn test_shift_q15_multiple_bits() {
                let src = [Q15::from_num(0.0625), Q15::from_num(0.03125)];
                let mut dst = [Q15::from_num(0.0); 2];
                <$T>::shift_q15(&src, 2, &mut dst);
                assert_eq!(dst, [Q15::from_num(0.25), Q15::from_num(0.125)]);
            }

            #[test]
            pub fn test_shift_in_place_q15_negative_values() {
                let mut values = [Q15::from_num(-0.5), Q15::from_num(-0.25)];
                <$T>::shift_in_place_q15(&mut values, 1);
                assert_eq!(values, [Q15::from_num(-1.0), Q15::from_num(-0.5)]);
            }
        }
    };
}
