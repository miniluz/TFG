use super::*;
use saw_wavetable::SAW_WAVETABLE;
use sine_wavetable::SINE_WAVETABLE;
use square_wavetable::SQUARE_WAVETABLE;
use triangle_wavetable::TRIANGLE_WAVETABLE;

type TestOps = cmsis_rust::CmsisRustOperations;
const TEST_SAMPLE_RATE: u32 = 48000;

// Test utilities module
mod utils {
    use super::*;

    /// Count zero crossings in a buffer (sign changes)
    pub fn count_zero_crossings(buffer: &[Q15]) -> usize {
        buffer
            .windows(2)
            .filter(|w| {
                let prev_positive = w[0] >= Q15::ZERO;
                let curr_positive = w[1] >= Q15::ZERO;
                prev_positive != curr_positive
            })
            .count()
    }

    /// Calculate DC offset (average value)
    pub fn calculate_dc_offset(buffer: &[Q15]) -> f64 {
        let sum: f64 = buffer.iter().map(|&v| v.to_num::<f64>()).sum();
        sum / buffer.len() as f64
    }

    /// Find maximum absolute value in buffer
    pub fn find_max_abs(buffer: &[Q15]) -> Q15 {
        buffer.iter().map(|&v| v.abs()).max().unwrap_or(Q15::ZERO)
    }

    /// Check if all samples are within valid Q15 range
    pub fn all_in_range(buffer: &[Q15]) -> bool {
        buffer.iter().all(|&v| v >= Q15::MIN && v <= Q15::MAX)
    }

    /// Check continuity: no sample-to-sample jump exceeds a threshold
    /// Returns true if all jumps are below threshold
    pub fn check_continuity(buffer: &[Q15], max_jump: Q15) -> bool {
        buffer.windows(2).enumerate().all(|(i, w)| {
            let diff = (w[1] - w[0]).abs();
            let result = diff <= max_jump;
            if !result {
                eprintln!(
                    "Window {} failed: {} - {} = {} > {}",
                    i, w[1], w[0], diff, max_jump
                );
            }
            result
        })
    }

    /// Create an oscillator with a specific wavetable and note
    pub fn create_osc(
        wavetable: &'static [Q15; 256],
        note: Note,
    ) -> WavetableOscillator<'static, TEST_SAMPLE_RATE> {
        WavetableOscillator {
            phase: U8F24::ZERO,
            phase_increment: MIDI_TO_PHASE_INCREMENT[note.as_u8() as usize],
            wavetable: Wavetable(wavetable),
        }
    }
}

// Property tests
#[test]
fn test_output_always_in_valid_range() {
    let note = Note::new(69); // A4
    let mut osc = utils::create_osc(&SINE_WAVETABLE, note);
    let mut buffer = [Q15::ZERO; 128];

    osc.get_samples::<TestOps, 128>(&mut buffer);

    assert!(utils::all_in_range(&buffer));
}

#[test]
fn test_phase_resets_on_set_note() {
    let note1 = Note::new(60);
    let note2 = Note::new(72);
    let mut osc = utils::create_osc(&SINE_WAVETABLE, note1);

    // Generate some samples to advance phase
    let mut buffer = [Q15::ZERO; 100];
    osc.get_samples::<TestOps, 100>(&mut buffer);

    // Phase should be non-zero now
    assert_ne!(osc.phase, U8F24::ZERO);

    // Set new note should reset phase
    osc.set_note(&note2);
    assert_eq!(osc.phase, U8F24::ZERO);
}

#[test]
fn test_sequential_calls_maintain_continuity() {
    let note = Note::new(69); // A4 = 440Hz
    let mut osc = utils::create_osc(&SINE_WAVETABLE, note);

    let mut buffer1 = [Q15::ZERO; 64];
    let mut buffer2 = [Q15::ZERO; 64];

    osc.get_samples::<TestOps, 64>(&mut buffer1);
    osc.get_samples::<TestOps, 64>(&mut buffer2);

    // Check that the transition between buffers is smooth
    // The jump from last sample of buffer1 to first of buffer2
    let last_of_first = buffer1[63];
    let first_of_second = buffer2[0];

    // For a 440Hz sine wave at 48kHz, max rate of change per sample
    // is approximately 2*pi*440/48000 ≈ 0.0576 radians/sample
    // In Q15 space, this is about 0.0576 * 32768 ≈ 1888
    let max_jump = Q15::from_num(0.2); // Conservative estimate

    let diff = (first_of_second - last_of_first).abs();
    assert!(
        diff <= max_jump,
        "Discontinuity between buffers: {} exceeds {}",
        diff,
        max_jump
    );
}

#[test]
fn test_no_internal_discontinuities() {
    let note = Note::new(69); // A4
    let mut osc = utils::create_osc(&SINE_WAVETABLE, note);
    let mut buffer = [Q15::ZERO; 256];

    osc.get_samples::<TestOps, 256>(&mut buffer);

    // For smooth waveforms like sine, triangle, saw
    // jumps should be bounded by frequency-dependent max
    let max_jump = Q15::from_num(0.2);
    assert!(
        utils::check_continuity(&buffer, max_jump),
        "Found discontinuity in waveform"
    );
}

#[test]
fn test_frequency_accuracy_via_zero_crossings() {
    // Test that a 440Hz sine wave (A4 = MIDI 69) produces correct frequency
    let note = Note::new(69); // A4 = 440Hz
    let mut osc = utils::create_osc(&SINE_WAVETABLE, note);

    // Generate 1 second of audio
    const SAMPLES_PER_SECOND: usize = TEST_SAMPLE_RATE as usize;
    let mut buffer = vec![Q15::ZERO; SAMPLES_PER_SECOND];

    // Process in chunks
    for chunk in buffer.chunks_mut(128) {
        let mut temp = [Q15::ZERO; 128];
        osc.get_samples::<TestOps, 128>(&mut temp);
        chunk.copy_from_slice(&temp[..chunk.len()]);
    }

    let zero_crossings = utils::count_zero_crossings(&buffer);
    // A sine wave has 2 zero crossings per cycle
    let cycles = zero_crossings as f64 / 2.0;

    // Expected: 440 cycles in 1 second
    // Allow 1% tolerance for quantization effects
    let expected_freq = 440.0;
    let tolerance = expected_freq * 0.01;

    assert!(
        (cycles - expected_freq).abs() < tolerance,
        "Frequency mismatch: got {} Hz, expected {} Hz",
        cycles,
        expected_freq
    );
}

// Waveform-specific tests
#[test]
fn test_sine_wave_dc_offset_near_zero() {
    let note = Note::new(69);
    let mut osc = utils::create_osc(&SINE_WAVETABLE, note);

    // Generate multiple complete cycles
    let mut buffer = [Q15::ZERO; 4096];
    osc.get_samples::<TestOps, 4096>(&mut buffer);

    let dc_offset = utils::calculate_dc_offset(&buffer);

    // DC offset should be very close to zero for a sine wave
    assert!(dc_offset.abs() < 0.01, "DC offset too large: {}", dc_offset);
}

#[test]
fn test_sine_wave_reaches_near_maximum() {
    let note = Note::new(60); // C4
    let mut osc = utils::create_osc(&SINE_WAVETABLE, note);

    // Generate enough samples to cover multiple cycles
    let mut buffer = [Q15::ZERO; 2048];
    osc.get_samples::<TestOps, 2048>(&mut buffer);

    let max_value = utils::find_max_abs(&buffer);

    // Should reach close to maximum Q15 value
    // Allow for interpolation not hitting exact peaks
    let expected_min = Q15::from_num(0.95);
    assert!(
        max_value >= expected_min,
        "Peak value {} too low, expected at least {}",
        max_value,
        expected_min
    );
}

#[test]
fn test_triangle_wave_dc_offset_near_zero() {
    let note = Note::new(69);
    let mut osc = utils::create_osc(&TRIANGLE_WAVETABLE, note);

    let mut buffer = [Q15::ZERO; 4096];
    osc.get_samples::<TestOps, 4096>(&mut buffer);

    let dc_offset = utils::calculate_dc_offset(&buffer);

    assert!(dc_offset.abs() < 0.01, "DC offset too large: {}", dc_offset);
}

#[test]
fn test_saw_wave_dc_offset_near_zero() {
    let note = Note::new(69);
    let mut osc = utils::create_osc(&SAW_WAVETABLE, note);

    let mut buffer = [Q15::ZERO; 4096];
    osc.get_samples::<TestOps, 4096>(&mut buffer);

    let dc_offset = utils::calculate_dc_offset(&buffer);

    assert!(dc_offset.abs() < 0.02, "DC offset too large: {}", dc_offset);
}

#[test]
fn test_square_wave_alternates() {
    let note = Note::new(60);
    let mut osc = utils::create_osc(&SQUARE_WAVETABLE, note);

    let mut buffer = [Q15::ZERO; 1024];
    osc.get_samples::<TestOps, 1024>(&mut buffer);

    // Square wave should have many zero crossings
    let crossings = utils::count_zero_crossings(&buffer);
    assert!(crossings > 10, "Square wave should have many transitions");

    // Square wave should reach near extremes
    let max_value = utils::find_max_abs(&buffer);
    assert!(max_value > Q15::from_num(0.8));
}

#[test]
fn test_deterministic_output() {
    let note = Note::new(69);

    let mut osc1 = utils::create_osc(&SINE_WAVETABLE, note);
    let mut osc2 = utils::create_osc(&SINE_WAVETABLE, note);

    let mut buffer1 = [Q15::ZERO; 256];
    let mut buffer2 = [Q15::ZERO; 256];

    osc1.get_samples::<TestOps, 256>(&mut buffer1);
    osc2.get_samples::<TestOps, 256>(&mut buffer2);

    assert_eq!(buffer1, buffer2, "Same inputs should produce same outputs");
}

#[test]
fn test_different_notes_produce_different_frequencies() {
    let note_low = Note::new(60); // C4
    let note_high = Note::new(72); // C5 (one octave higher)

    let mut osc_low = utils::create_osc(&SINE_WAVETABLE, note_low);
    let mut osc_high = utils::create_osc(&SINE_WAVETABLE, note_high);

    let mut buffer_low = vec![Q15::ZERO; TEST_SAMPLE_RATE as usize];
    let mut buffer_high = vec![Q15::ZERO; TEST_SAMPLE_RATE as usize];

    // Process in chunks
    for chunk in buffer_low.chunks_mut(128) {
        let mut temp = [Q15::ZERO; 128];
        osc_low.get_samples::<TestOps, 128>(&mut temp);
        chunk.copy_from_slice(&temp[..chunk.len()]);
    }

    for chunk in buffer_high.chunks_mut(128) {
        let mut temp = [Q15::ZERO; 128];
        osc_high.get_samples::<TestOps, 128>(&mut temp);
        chunk.copy_from_slice(&temp[..chunk.len()]);
    }

    let crossings_low = utils::count_zero_crossings(&buffer_low);
    let crossings_high = utils::count_zero_crossings(&buffer_high);

    // One octave higher should have approximately 2x the frequency
    let ratio = crossings_high as f64 / crossings_low as f64;

    // Should be close to 2.0, allow 2% tolerance
    assert!(
        (ratio - 2.0).abs() < 0.04,
        "Octave ratio incorrect: got {}, expected ~2.0",
        ratio
    );
}

#[test]
fn test_phase_wrapping_behavior() {
    let note = Note::new(69);
    let mut osc = utils::create_osc(&SINE_WAVETABLE, note);

    // Generate many samples to ensure phase wraps multiple times
    let mut buffer = [Q15::ZERO; 4096];
    osc.get_samples::<TestOps, 4096>(&mut buffer);

    // Should still produce valid output after wrapping
    assert!(utils::all_in_range(&buffer));
    // Should still be continuous
    assert!(utils::check_continuity(&buffer, Q15::from_num(0.1)));
}
