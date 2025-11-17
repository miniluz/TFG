#![no_std]

pub use fixed::types::I1F15 as Q15;

pub trait CmsisOperations {
    fn abs_in_place_q15(values: &mut [Q15]);
    fn abs_q15(src: &[Q15], dst: &mut [Q15]);
    fn add_q15(src1: &[Q15], src2: &[Q15], dst: &mut [Q15]);
    fn multiply_q15(src1: &[Q15], src2: &[Q15], dst: &mut [Q15]);
    fn negate_in_place_q15(values: &mut [Q15]);
    fn negate_q15(src: &[Q15], dst: &mut [Q15]);
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
        }
    };
}
