#![cfg_attr(not(test), no_std)]

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn multiply_f32(src1: &[f32], src2: &[f32], dst: &mut [f32]) {
    #[cfg(test)]
    {
        if (src1.len() != src2.len()) || (src1.len() != dst.len()) {
            panic!("src1.len() != src2.len() || src1.len() != dst.len()");
        }

        for (dst, (src1, src2)) in dst.iter_mut().zip(src1.iter().zip(src2.iter())) {
            *dst = *src1 * *src2;
        }
    }

    #[cfg(not(test))]
    {
        use cmsis_dsp::basic::multiply_f32;

        multiply_f32(src1, src2, dst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_add_it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn when_multiply_f32_it_works() {
        let src1 = [1.0, 2.0, 3.0];
        let src2 = [1.0, 2.0, 3.0];
        let mut dst = [0.0; 3];
        multiply_f32(&src1, &src2, &mut dst);
        assert_eq!(dst, [1.0, 4.0, 9.0]);
    }
}
