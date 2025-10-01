#![no_std]
#![no_main]

use adder_interface::AddOperations;

pub struct CmsisAddOperations;

impl AddOperations for CmsisAddOperations {
    fn add(left: u64, right: u64) -> u64 {
        left + right
    }

    fn multiply_f32(src1: &[f32], src2: &[f32], dst: &mut [f32]) {
        use cmsis_dsp::basic::multiply_f32;

        multiply_f32(src1, src2, dst);
    }
}

#[cfg(test)]
#[embedded_test::setup]
fn setup() {
    rtt_target::rtt_init_defmt!();
}

#[cfg(test)]
adder_interface::declare_tests! {
    crate::CmsisAddOperations,
    #[embedded_test::tests],
    use embassy_stm32 as _;
}
