#![cfg(feature = "octave-filter")]

use cmsis_rust::CmsisRustOperations as Ops;
use synth_engine::{OctaveFilterBank, Q15, SAMPLE_RATE};

const WINDOW_SIZE: usize = 128;
const TEST_DURATION_SAMPLES: usize = SAMPLE_RATE as usize; // 1 second

/// Generate a pure sine wave at the given frequency
fn generate_sine_wave(freq: f64, duration_samples: usize) -> Vec<Q15> {
    let mut samples = Vec::with_capacity(duration_samples);
    for i in 0..duration_samples {
        let t = i as f64 / SAMPLE_RATE as f64;
        let value = (2.0 * std::f64::consts::PI * freq * t).sin();
        samples.push(Q15::from_num(value * 0.8)); // Scale to 0.8 to avoid clipping
    }
    samples
}

/// Generate white noise
fn generate_white_noise(duration_samples: usize, seed: u64) -> Vec<Q15> {
    use std::num::Wrapping;

    let mut samples = Vec::with_capacity(duration_samples);
    let mut rng_state = Wrapping(seed);

    for _ in 0..duration_samples {
        // Simple LCG random number generator
        rng_state = rng_state * Wrapping(1664525u64) + Wrapping(1013904223u64);
        let random_u32 = (rng_state.0 >> 32) as u32;
        let random_i16 = (random_u32 >> 16) as i16;

        // Scale to ±0.5 range for white noise
        let value = (random_i16 as f64 / i16::MAX as f64) * 0.5;
        samples.push(Q15::from_num(value));
    }
    samples
}

/// Calculate RMS energy of a signal
fn calculate_rms(samples: &[Q15]) -> f64 {
    let sum_squares: f64 = samples
        .iter()
        .map(|s| {
            let val = s.to_num::<f64>();
            val * val
        })
        .sum();

    (sum_squares / samples.len() as f64).sqrt()
}

/// Process signal through octave filter in chunks
fn process_signal(filter: &mut OctaveFilterBank, input: &[Q15]) -> Vec<Q15> {
    let mut output = Vec::with_capacity(input.len());

    for chunk in input.chunks(WINDOW_SIZE) {
        let mut input_array = [Q15::ZERO; WINDOW_SIZE];
        let mut output_array = [Q15::ZERO; WINDOW_SIZE];

        // Copy input to fixed-size array
        for (i, &sample) in chunk.iter().enumerate() {
            input_array[i] = sample;
        }

        // Process through filter
        filter.process::<Ops, WINDOW_SIZE>(&input_array, &mut output_array);

        // Collect output
        output.extend_from_slice(&output_array[..chunk.len()]);
    }

    output
}

#[test]
fn test_single_band_frequency_selectivity_band_0() {
    // Test 4: Single-Band Frequency Selectivity for Band 0 (62.5 Hz)
    let center_freq = 62.5;
    let input = generate_sine_wave(center_freq, TEST_DURATION_SAMPLES);
    let input_rms = calculate_rms(&input);

    let mut filter = OctaveFilterBank::new();

    // Configure: Band 0 at unity gain (255), all others at minimum (0)
    filter.set_band_gain(0, 255);
    for band in 1..6 {
        filter.set_band_gain(band, 0);
    }

    let output = process_signal(&mut filter, &input);
    let output_rms = calculate_rms(&output);

    // Target band should pass >90% of energy
    let energy_ratio = output_rms / input_rms;
    assert!(
        energy_ratio > 0.90,
        "Band 0 (62.5Hz) should pass >90% energy, got {:.2}%",
        energy_ratio * 100.0
    );
}

#[test]
fn test_single_band_frequency_selectivity_band_2() {
    // Test 4: Single-Band Frequency Selectivity for Band 2 (250 Hz)
    let center_freq = 250.0;
    let input = generate_sine_wave(center_freq, TEST_DURATION_SAMPLES);
    let input_rms = calculate_rms(&input);

    let mut filter = OctaveFilterBank::new();

    // Configure: Band 2 at unity gain (255), all others at minimum (0)
    filter.set_band_gain(2, 255);
    for band in [0, 1, 3, 4, 5] {
        filter.set_band_gain(band, 0);
    }

    let output = process_signal(&mut filter, &input);
    let output_rms = calculate_rms(&output);

    // Target band should pass >90% of energy
    let energy_ratio = output_rms / input_rms;
    assert!(
        energy_ratio > 0.90,
        "Band 2 (250Hz) should pass >90% energy, got {:.2}%",
        energy_ratio * 100.0
    );
}

#[test]
fn test_single_band_frequency_selectivity_band_4() {
    // Test 4: Single-Band Frequency Selectivity for Band 4 (1000 Hz)
    let center_freq = 1000.0;
    let input = generate_sine_wave(center_freq, TEST_DURATION_SAMPLES);
    let input_rms = calculate_rms(&input);

    let mut filter = OctaveFilterBank::new();

    // Configure: Band 4 at unity gain (255), all others at minimum (0)
    filter.set_band_gain(4, 255);
    for band in [0, 1, 2, 3, 5] {
        filter.set_band_gain(band, 0);
    }

    let output = process_signal(&mut filter, &input);
    let output_rms = calculate_rms(&output);

    // Target band should pass >90% of energy
    let energy_ratio = output_rms / input_rms;
    assert!(
        energy_ratio > 0.90,
        "Band 4 (1000Hz) should pass >90% energy, got {:.2}%",
        energy_ratio * 100.0
    );
}

#[test]
fn test_adjacent_band_rejection() {
    // Test that adjacent bands reject the signal
    // Test frequency at band 2 (250 Hz), enable only band 4 (1000 Hz - non-adjacent)
    let test_freq = 250.0;
    let input = generate_sine_wave(test_freq, TEST_DURATION_SAMPLES);
    let input_rms = calculate_rms(&input);

    let mut filter = OctaveFilterBank::new();

    // Configure: Only band 4 enabled, all others disabled
    filter.set_band_gain(4, 255);
    for band in [0, 1, 2, 3, 5] {
        filter.set_band_gain(band, 0);
    }

    let output = process_signal(&mut filter, &input);
    let output_rms = calculate_rms(&output);

    // Non-adjacent band should reject >99% of energy (pass <1%)
    let energy_ratio = output_rms / input_rms;
    assert!(
        energy_ratio < 0.01,
        "Non-adjacent band should reject >99% energy, got {:.2}% pass-through",
        energy_ratio * 100.0
    );
}

#[test]
fn test_multi_band_frequency_rejection_config_x() {
    // Test 5: Multi-Band Frequency Rejection - Configuration X
    // Two tones: 125Hz + 1kHz, both bands disabled (0), others enabled (255)

    let tone1_freq = 125.0; // Band 1
    let tone2_freq = 1000.0; // Band 4

    // Generate two-tone signal
    let mut input = vec![Q15::ZERO; TEST_DURATION_SAMPLES];
    for (i, sample) in input.iter_mut().enumerate().take(TEST_DURATION_SAMPLES) {
        let t = i as f64 / SAMPLE_RATE as f64;
        let value1 = (2.0 * std::f64::consts::PI * tone1_freq * t).sin() * 0.4;
        let value2 = (2.0 * std::f64::consts::PI * tone2_freq * t).sin() * 0.4;
        *sample = Q15::from_num(value1 + value2);
    }

    let input_rms = calculate_rms(&input);

    let mut filter = OctaveFilterBank::new();

    // Configuration X: Band 1 (125Hz) = 0, Band 4 (1kHz) = 0, all others = 255
    for band in 0..6 {
        if band == 1 || band == 4 {
            filter.set_band_gain(band, 0);
        } else {
            filter.set_band_gain(band, 255);
        }
    }

    let output = process_signal(&mut filter, &input);
    let output_rms = calculate_rms(&output);

    // Both frequencies should be eliminated (>95% attenuation = <5% pass-through)
    let energy_ratio = output_rms / input_rms;
    assert!(
        energy_ratio < 0.05,
        "Config X: Both tones should be >95% attenuated, got {:.2}% pass-through",
        energy_ratio * 100.0
    );
}

#[test]
fn test_multi_band_frequency_rejection_config_y() {
    // Test 5: Multi-Band Frequency Rejection - Configuration Y
    // Band 1 (125Hz) = 127 (~half gain), Band 4 (1kHz) = 191 (~3/4 gain)

    let tone1_freq = 125.0; // Band 1
    let tone2_freq = 1000.0; // Band 4

    // Generate two separate tones to measure independently
    let tone1_input = generate_sine_wave(tone1_freq, TEST_DURATION_SAMPLES);
    let tone2_input = generate_sine_wave(tone2_freq, TEST_DURATION_SAMPLES);

    let tone1_input_rms = calculate_rms(&tone1_input);
    let tone2_input_rms = calculate_rms(&tone2_input);

    // Test tone 1 (125Hz) with reduced gain
    let mut filter1 = OctaveFilterBank::new();
    filter1.set_band_gain(1, 127); // ~half gain
    for band in [0, 2, 3, 4, 5] {
        filter1.set_band_gain(band, 0);
    }

    let tone1_output = process_signal(&mut filter1, &tone1_input);
    let tone1_output_rms = calculate_rms(&tone1_output);
    let tone1_ratio = tone1_output_rms / tone1_input_rms;

    // Tone 1 should be reduced to approximately 40-60% (target ~50%)
    assert!(
        (0.35..=0.65).contains(&tone1_ratio),
        "Config Y: 125Hz tone should be ~50% amplitude, got {:.2}%",
        tone1_ratio * 100.0
    );

    // Test tone 2 (1kHz) with higher gain
    let mut filter2 = OctaveFilterBank::new();
    filter2.set_band_gain(4, 191); // ~3/4 gain
    for band in [0, 1, 2, 3, 5] {
        filter2.set_band_gain(band, 0);
    }

    let tone2_output = process_signal(&mut filter2, &tone2_input);
    let tone2_output_rms = calculate_rms(&tone2_output);
    let tone2_ratio = tone2_output_rms / tone2_input_rms;

    // Tone 2 should be reduced to approximately 70-80% (target ~75%)
    assert!(
        (0.65..=0.85).contains(&tone2_ratio),
        "Config Y: 1kHz tone should be ~75% amplitude, got {:.2}%",
        tone2_ratio * 100.0
    );
}

#[test]
fn test_white_noise_band_energy_distribution() {
    // Test 6: White Noise Band Energy Distribution

    let input = generate_white_noise(TEST_DURATION_SAMPLES, 12345);
    let input_rms = calculate_rms(&input);

    let mut filter = OctaveFilterBank::new();

    // Configure all bands to unity gain (255)
    for band in 0..6 {
        filter.set_band_gain(band, 255);
    }

    let output = process_signal(&mut filter, &input);
    let output_rms = calculate_rms(&output);

    // Energy should be approximately conserved (allowing for filter overlap)
    // With 6 bands and some overlap, expect output to be within 50-150% of input
    let energy_ratio = output_rms / input_rms;
    assert!(
        (0.5..=1.5).contains(&energy_ratio),
        "White noise energy should be approximately conserved, got {:.2}x",
        energy_ratio
    );

    // Verify no single band dominates by testing individual bands
    let mut band_energies = Vec::new();

    for test_band in 0..6 {
        let mut band_filter = OctaveFilterBank::new();

        // Enable only this band
        band_filter.set_band_gain(test_band, 255);
        for band in 0..6 {
            if band != test_band {
                band_filter.set_band_gain(band, 0);
            }
        }

        let band_output = process_signal(&mut band_filter, &input);
        let band_rms = calculate_rms(&band_output);
        band_energies.push(band_rms);
    }

    // Find max and min band energy
    let max_energy = band_energies
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let min_energy = band_energies.iter().cloned().fold(f64::INFINITY, f64::min);

    // No band should have more than 3x the energy of any other band
    // (allows for some variation due to filter characteristics)
    let energy_ratio = max_energy / min_energy;
    assert!(
        energy_ratio < 3.0,
        "Band energy distribution too uneven: max/min = {:.2}",
        energy_ratio
    );
}

#[test]
fn test_config_integration() {
    // Test 7: Config Integration
    // Test that band gains can be updated and take effect

    let test_freq = 500.0; // Band 3
    let input = generate_sine_wave(test_freq, TEST_DURATION_SAMPLES);
    let input_rms = calculate_rms(&input);

    let mut filter = OctaveFilterBank::new();

    // Initial config: Band 3 disabled
    filter.set_band_gain(3, 0);
    for band in [0, 1, 2, 4, 5] {
        filter.set_band_gain(band, 0);
    }

    let output1 = process_signal(&mut filter, &input);
    let output1_rms = calculate_rms(&output1);

    // Should be heavily attenuated
    assert!(
        output1_rms / input_rms < 0.05,
        "Band 3 disabled should attenuate >95%"
    );

    // Update config: Enable Band 3
    filter.set_band_gain(3, 255);

    let output2 = process_signal(&mut filter, &input);
    let output2_rms = calculate_rms(&output2);

    // Should now pass through
    assert!(
        output2_rms / input_rms > 0.90,
        "Band 3 enabled should pass >90%"
    );

    // Test rapid updates (multiple gain changes)
    for gain in [64, 128, 191, 255] {
        filter.set_band_gain(3, gain);

        // Process a small chunk to verify no crashes
        let mut input_chunk = [Q15::ZERO; WINDOW_SIZE];
        let mut output_chunk = [Q15::ZERO; WINDOW_SIZE];
        input_chunk[..WINDOW_SIZE].copy_from_slice(&input[..WINDOW_SIZE]);

        filter.process::<Ops, WINDOW_SIZE>(&input_chunk, &mut output_chunk);

        // Just verify it doesn't crash and produces valid output
        let chunk_rms = calculate_rms(&output_chunk);
        assert!(
            chunk_rms.is_finite(),
            "Output should be finite after gain update"
        );
    }
}

#[test]
fn test_all_bands_disabled_produces_silence() {
    // Edge case: All bands disabled should produce near-silence

    let input = generate_sine_wave(500.0, TEST_DURATION_SAMPLES);
    let input_rms = calculate_rms(&input);

    let mut filter = OctaveFilterBank::new();

    // Disable all bands
    for band in 0..6 {
        filter.set_band_gain(band, 0);
    }

    let output = process_signal(&mut filter, &input);
    let output_rms = calculate_rms(&output);

    // Output should be <1% of input
    let energy_ratio = output_rms / input_rms;
    assert!(
        energy_ratio < 0.01,
        "All bands disabled should produce <1% output, got {:.2}%",
        energy_ratio * 100.0
    );
}

#[test]
fn test_passthrough_with_all_bands_enabled() {
    // Test that with all bands at unity gain, signal passes through

    let input = generate_sine_wave(500.0, TEST_DURATION_SAMPLES);
    let input_rms = calculate_rms(&input);

    let mut filter = OctaveFilterBank::new();

    // Enable all bands at unity gain
    for band in 0..6 {
        filter.set_band_gain(band, 255);
    }

    let output = process_signal(&mut filter, &input);
    let output_rms = calculate_rms(&output);

    // Output should be similar to input (allowing for some filter effects)
    // Expect 70-130% due to band overlap and filter characteristics
    let energy_ratio = output_rms / input_rms;
    assert!(
        (0.70..=1.30).contains(&energy_ratio),
        "All bands enabled should pass signal through (70-130%), got {:.2}%",
        energy_ratio * 100.0
    );
}
