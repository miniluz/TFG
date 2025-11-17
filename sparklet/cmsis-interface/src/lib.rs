#![no_std]

pub trait CmsisOperations {
    fn add(left: u64, right: u64) -> u64;

    fn multiply_f32(src1: &[f32], src2: &[f32], dst: &mut [f32]);
}

#[macro_export]
macro_rules! declare_tests {
    {$T:ty, $(#[$meta:meta]),*, $($prelude:tt)*} => {
        #[cfg(test)]
        $(#[$meta])*
        mod tests {

            $($prelude)*

            use cmsis_interface::CmsisOperations;

            #[test]
            pub fn when_add_it_works() {
                let result = <$T>::add(2, 2);
                assert_eq!(result, 4);
            }

            #[test]
            pub fn when_multiply_f32_it_works() {
                let src1 = [1.0, 2.0, 3.0];
                let src2 = [1.0, 2.0, 3.0];
                let mut dst = [0.0; 3];
                <$T>::multiply_f32(&src1, &src2, &mut dst);
                assert_eq!(dst, [1.0, 4.0, 9.0]);
            }
        }
    };
}
