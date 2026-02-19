#![no_std]

pub use fixed::types::I1F15 as Q15;

/// State for Q15 Biquad cascade filter
/// The state array has length 4*NUM_STAGES and stores [x[n-1], x[n-2], y[n-1], y[n-2]] for each stage
#[derive(Clone, Copy, Debug)]
pub struct BiquadCascadeDf1StateQ15<const NUM_STAGES: usize, const STATE_SIZE: usize> {
    state: [Q15; STATE_SIZE],
}

impl<const NUM_STAGES: usize, const STATE_SIZE: usize>
    BiquadCascadeDf1StateQ15<NUM_STAGES, STATE_SIZE>
{
    pub const fn new() -> Self {
        assert!(
            STATE_SIZE == NUM_STAGES * 4,
            "STATE_SIZE must equal NUM_STAGES * 4"
        );
        Self {
            state: [Q15::ZERO; STATE_SIZE],
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [Q15] {
        &mut self.state
    }

    pub fn as_slice(&self) -> &[Q15] {
        &self.state
    }
}

impl<const NUM_STAGES: usize, const STATE_SIZE: usize> Default
    for BiquadCascadeDf1StateQ15<NUM_STAGES, STATE_SIZE>
{
    fn default() -> Self {
        Self::new()
    }
}

pub trait CmsisOperations {
    fn abs_in_place_q15(values: &mut [Q15]);
    fn abs_q15(src: &[Q15], dst: &mut [Q15]);
    fn add_q15(src1: &[Q15], src2: &[Q15], dst: &mut [Q15]);
    fn multiply_q15(src1: &[Q15], src2: &[Q15], dst: &mut [Q15]);
    fn negate_in_place_q15(values: &mut [Q15]);
    fn negate_q15(src: &[Q15], dst: &mut [Q15]);
    fn shift_q15(src: &[Q15], shift_bits: i8, dst: &mut [Q15]);
    fn shift_in_place_q15(values: &mut [Q15], shift_bits: i8);

    fn biquad_cascade_df1_q15<const NUM_STAGES: usize, const STATE_SIZE: usize>(
        state: &mut BiquadCascadeDf1StateQ15<NUM_STAGES, STATE_SIZE>,
        coeffs: &[[Q15; 6]; NUM_STAGES],
        post_shift: i8,
        input: &[Q15],
        output: &mut [Q15],
    );
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

            #[test]
            pub fn test_biquad_cascade_passthrough() {
                use cmsis_interface::BiquadCascadeDf1StateQ15;

                let b0 = Q15::from_num(0.125);
                let b1 = Q15::from_num(0.25);
                let b2 = Q15::from_num(0.0675);

                let a1 = Q15::from_num(0.5);
                let a2 = Q15::from_num(0.25);

                // CMSIS format: [b0, 0, b1, b2, a1, a2]
                let coeffs = [[b0, Q15::ZERO, b1, b2, a1, a2]];
                let mut state = BiquadCascadeDf1StateQ15::<1, 4>::default();

                let input1 = [Q15::from_num(0.5), Q15::from_num(-0.25), Q15::from_num(0.75), Q15::from_num(0.5)];
                let mut output1 = [Q15::ZERO; 4];
                <$T>::biquad_cascade_df1_q15::<1, 4>(&mut state, &coeffs, 0, &input1, &mut output1);

                let max_bits = Q15::MAX.to_bits() as i64;
                let e1 = b0 * input1[0];
                let e2 = a1 * e1 + b0 * input1[1] + b1 * input1[0];
                let e3 = a1 * e2 + a2 * e1 + b0 * input1[2] + b1 * input1[1] + b2 * input1[0];
                let e4 = a1 * e3 + a2 * e2 + b0 * input1[3] + b1 * input1[2] + b2 * input1[1];

                assert!((output1[0] - e1).abs() < Q15::from_num(0.001));
                assert!((output1[1] - e2).abs() < Q15::from_num(0.001));
                assert!((output1[2] - e3).abs() < Q15::from_num(0.001));
                assert!((output1[3] - e4).abs() < Q15::from_num(0.001));

                let input2 = [Q15::from_num(0.5), Q15::from_num(-0.25), Q15::from_num(0.75), Q15::from_num(0.5)];
                let mut output2 = [Q15::ZERO; 4];
                <$T>::biquad_cascade_df1_q15::<1, 4>(&mut state, &coeffs, 0, &input2, &mut output2);

                let e5 = a1 * e4 + a2 * e3 + b0 * input2[0] + b1 * input1[3] + b2 * input1[2];
                let e6 = a1 * e5 + a2 * e4 + b0 * input2[1] + b1 * input2[0] + b2 * input1[3];
                let e7 = a1 * e6 + a2 * e5 + b0 * input2[2] + b1 * input2[1] + b2 * input2[0];
                let e8 = a1 * e7 + a2 * e6 + b0 * input2[3] + b1 * input2[2] + b2 * input2[1];

                assert!((output2[0] - e5).abs() < Q15::from_num(0.001));
                assert!((output2[1] - e6).abs() < Q15::from_num(0.001));
                assert!((output2[2] - e7).abs() < Q15::from_num(0.001));
                assert!((output2[3] - e8).abs() < Q15::from_num(0.001));

            }
        }
    };
}
