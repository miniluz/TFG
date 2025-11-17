use cmsis_interface::{CmsisOperations, Q15};
use fixed::types::U8F24;

pub struct Wavetable([Q15; 256]);

pub struct WavetableOscillator<const SAMPLE_RATE: u32> {
    phase: U8F24,
    phase_increment: U8F24,
    wavetable: Wavetable,
}

impl<const SAMPLE_RATE: u32> WavetableOscillator<SAMPLE_RATE> {
    pub fn get_samples<T: CmsisOperations, const LEN: usize>(
        self: &mut Self,
        buffer: &mut [Q15; LEN],
    ) {
        /*
         * We're gonna use SIMD to calculate for efficienty
         * So we'll be collecting things into arrays first
         *
         * Output will be interpolated between two adjacent wavetable samples
         * and blended based on the fractional part of the phase
         */

        let mut sample_current = [Q15::ZERO; LEN];
        let mut sample_next = [Q15::ZERO; LEN];
        let mut weight_current = [Q15::ZERO; LEN];
        let mut weight_next = [Q15::ZERO; LEN];

        for (s_current, (s_next, (w_next))) in sample_current
            .iter_mut()
            .zip(sample_next.iter_mut().zip(weight_next.iter_mut()))
        {
            // Use the integer part (first 8 bits) as the index
            let index: u8 = self.phase.to_num();

            *s_current = self.wavetable.0[index as usize];
            *s_next = self.wavetable.0[index.wrapping_add(1) as usize];

            /*
             * We're gonna convert the fractional part of the phase into Q15
             * with cursed bit magic
             *
             * The representation looks like:
             *
             * | -------- | --------------- | ---------- |
             * | Index    | Interpolation   | Extra      |
             * | -------- | --------------- | ---------- |
             * | bits 0-7 | bits 8-22       | bits 23-31 |
             * | 00000000 . 111111111111111   000000000  |
             * | -------- | --------------- | ---------- |
             *
             * Bits 8-22 are the first 15 fractional bits.
             * To convert to I1F15 (Q15), we can shift right
             * and add a 0 at the beginning beacuse the number
             * is always positive
             *
             *       bits 8-22
             *  0.111111111111111
             *
             *  Thus we can shift right by 9 bits, cast to i16,
             * and AND with 0x7FFF.
             */

            let bits = self.phase.to_bits();
            let weight_aligned = bits.unbounded_shr(9) as u16;
            let weight_masked = weight_aligned & 0x7FFF;

            *w_next = Q15::from_bits(weight_masked as i16);

            self.phase = self.phase.wrapping_add(self.phase_increment);
        }

        // w_current = MAX - w_next
        // Using buffer here might seem odd
        // but since we're gonna override it anyways who cares...
        T::negate_q15(&weight_next, buffer);
        T::add_q15(buffer, &[Q15::MAX; LEN], &mut weight_current);

        // output = s_current * w_current + s_next * w_next
        // TODO: Maybe optimize this further,
        // .clone() is probably less efficient than creating an empty buffer
        // but who knows if the compiler will optimize it away...
        T::multiply_q15(
            &sample_current.clone(),
            &weight_current,
            &mut sample_current,
        );
        T::multiply_q15(&sample_next.clone(), &weight_next, &mut sample_next);

        T::add_q15(&sample_current, &sample_next, buffer);
    }
}
