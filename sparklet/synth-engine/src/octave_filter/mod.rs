pub mod filter_coefficients;

use crate::adsr::db_linear_amplitude_table::DB_LINEAR_AMPLITUDE_TABLE;
use cmsis_interface::{BiquadCascadeDf1StateQ15, CmsisOperations, Q15};
use filter_coefficients::{OCTAVE_FILTER_COEFFS, OCTAVE_FILTER_POST_SHIFT};

pub struct OctaveFilterBank {
    states: [BiquadCascadeDf1StateQ15<1, 4>; 6],
    band_gains: [Q15; 6],
}

impl OctaveFilterBank {
    pub fn new() -> Self {
        let default_gain = Q15::from_bits((DB_LINEAR_AMPLITUDE_TABLE[200].to_bits() >> 16) as i16);
        Self {
            states: [BiquadCascadeDf1StateQ15::new(); 6],
            band_gains: [default_gain; 6],
        }
    }

    pub fn set_band_gain(&mut self, band: usize, encoder_value: u8) {
        assert!(band < 6, "Band index must be 0-5");

        let i1f31_gain = DB_LINEAR_AMPLITUDE_TABLE[encoder_value as usize];
        self.band_gains[band] = Q15::from_bits((i1f31_gain.to_bits() >> 16) as i16);
    }

    pub fn process_one_band<T: CmsisOperations, const WINDOW_SIZE: usize>(
        &mut self,
        input: &[Q15; WINDOW_SIZE],
        output: &mut [Q15; WINDOW_SIZE],
        band_index: usize,
    ) {
        T::biquad_cascade_df1_q15(
            &mut self.states[band_index],
            &[OCTAVE_FILTER_COEFFS[band_index]],
            OCTAVE_FILTER_POST_SHIFT,
            input,
            output,
        );
    }

    pub fn process<T: CmsisOperations, const WINDOW_SIZE: usize>(
        &mut self,
        input: &[Q15; WINDOW_SIZE],
        output: &mut [Q15; WINDOW_SIZE],
    ) {
        let mut filtered = [Q15::ZERO; WINDOW_SIZE];
        let mut scaled = [Q15::ZERO; WINDOW_SIZE];
        let mut temp_output = [Q15::ZERO; WINDOW_SIZE];
        output.copy_from_slice(&temp_output);

        for band_index in 0..6 {
            self.process_one_band::<T, WINDOW_SIZE>(input, &mut filtered, band_index);

            let gain_array = [self.band_gains[band_index]; WINDOW_SIZE];
            T::multiply_q15(&filtered, &gain_array, &mut scaled);

            // Accumulate to output using saturating addition
            if band_index.is_multiple_of(2) {
                // First time, we copy to temp output
                T::add_q15(output, &scaled, &mut temp_output);
            } else {
                // Every other time, we copy back to output
                T::add_q15(&temp_output, &scaled, output);
            }
        }

        if !self.band_gains.len().is_multiple_of(2) {
            // If we did it an odd amount of times, the final result is in temp_output.
            output.copy_from_slice(&temp_output);
        }
    }
}

impl Default for OctaveFilterBank {
    fn default() -> Self {
        Self::new()
    }
}
