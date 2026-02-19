use cmsis_rust::CmsisRustOperations as Ops;
use std::f64::consts::PI;
use synth_engine::{OctaveFilterBank, Q15, SAMPLE_RATE};

const WINDOW_SIZE: usize = 256;

fn generate_sine_wave(freq: f64, num_samples: usize) -> Vec<Q15> {
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f64 / SAMPLE_RATE as f64;
        let value = (2.0 * PI * freq * t).sin();
        samples.push(Q15::saturating_from_num(value));
    }
    samples
}

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

fn process_signal_for_band(
    filter: &mut OctaveFilterBank,
    input: &[Q15],
    band_index: usize,
) -> Vec<Q15> {
    let mut output = Vec::with_capacity(input.len());

    for chunk in input.chunks(WINDOW_SIZE) {
        let mut input_array = [Q15::ZERO; WINDOW_SIZE];
        let mut output_array = [Q15::ZERO; WINDOW_SIZE];

        for (i, &sample) in chunk.iter().enumerate() {
            input_array[i] = sample;
        }

        filter.process_one_band::<Ops, WINDOW_SIZE>(&input_array, &mut output_array, band_index);

        output.extend_from_slice(&output_array[..chunk.len()]);
    }

    output
}

fn process_signal(filter: &mut OctaveFilterBank, input: &[Q15]) -> Vec<Q15> {
    let mut output = Vec::with_capacity(input.len());

    for chunk in input.chunks(WINDOW_SIZE) {
        let mut input_array = [Q15::ZERO; WINDOW_SIZE];
        let mut output_array = [Q15::ZERO; WINDOW_SIZE];

        for (i, &sample) in chunk.iter().enumerate() {
            input_array[i] = sample;
        }

        filter.process::<Ops, WINDOW_SIZE>(&input_array, &mut output_array);

        output.extend_from_slice(&output_array[..chunk.len()]);
    }

    output
}

fn main() {
    println!("Octave Filter Bank Frequency Response Analysis");
    println!("===============================================\n");

    let test_duration = SAMPLE_RATE as usize; // 1 second

    let test_freqs = [
        10.0, 20.0, 30.0, 40.0, 50.0, 62.5, 80.0, 100.0, 125.0, 160.0, 200.0, 250.0, 315.0, 400.0,
        500.0, 630.0, 800.0, 1000.0, 1250.0, 1600.0, 2000.0, 2500.0, 3150.0, 4000.0, 6000.0,
        8000.0, 10000.0, 14000.0, 18000.0, 20000.0, 22000.0,
    ];

    println!("Test 1: Raw DT1 Response");
    println!();
    println!("Freq (Hz) | Band 0 | Band 1 | Band 2 | Band 3 | Band 4 | Band 5");
    println!("----------|--------|--------|--------|--------|--------|--------");

    for &freq in &test_freqs {
        print!("{:9.1} |", freq);

        let mut filter = OctaveFilterBank::new();

        for band_index in 0..6 {
            let input = generate_sine_wave(freq, WINDOW_SIZE);
            let input_rms = calculate_rms(&input);

            let output = process_signal_for_band(&mut filter, &input, band_index);

            let output_rms = calculate_rms(&output);

            let gain_db = 20.0 * (output_rms / input_rms).log10();
            print!(" {:6.2} |", gain_db);
        }
        println!();
    }

    println!("\nTest 2: All bands enabled (default gain)");
    println!("--------------------------------------------");
    println!("Freq (Hz) | Gain (dB)");
    println!("----------|-----------");

    for &freq in &test_freqs {
        let input = generate_sine_wave(freq, test_duration);
        let input_rms = calculate_rms(&input);

        let mut filter = OctaveFilterBank::new();

        let output = process_signal(&mut filter, &input);
        let output_rms = calculate_rms(&output);

        let gain_ratio = output_rms / input_rms;
        let gain_db = 20.0 * gain_ratio.log10();

        println!("{:9.1} | {:9.2} ", freq, gain_db);
    }

    println!("\nTest 3: Odd bands maxed");
    println!("--------------------------------------------");
    println!("Freq (Hz) | Gain (dB) ");
    println!("----------|-----------");

    for &freq in &test_freqs {
        let input = generate_sine_wave(freq, test_duration);
        let input_rms = calculate_rms(&input);

        let mut filter = OctaveFilterBank::new();

        for band in 0..6_usize {
            if band.is_multiple_of(2) {
                filter.set_band_gain(band, 0);
            } else {
                filter.set_band_gain(band, 255);
            }
        }

        let output = process_signal(&mut filter, &input);
        let output_rms = calculate_rms(&output);

        let gain_ratio = output_rms / input_rms;
        let gain_db = 20.0 * gain_ratio.log10();

        println!("{:9.1} | {:9.2}", freq, gain_db);
    }

    println!("\nTest 4: Even bands maxed");
    println!("--------------------------------------------");
    println!("Freq (Hz) | Gain (dB)");
    println!("----------|-----------");

    for &freq in &test_freqs {
        let input = generate_sine_wave(freq, test_duration);
        let input_rms = calculate_rms(&input);

        let mut filter = OctaveFilterBank::new();

        for band in 0..6_usize {
            if !band.is_multiple_of(2) {
                filter.set_band_gain(band, 0);
            } else {
                filter.set_band_gain(band, 255);
            }
        }

        let output = process_signal(&mut filter, &input);
        let output_rms = calculate_rms(&output);

        let gain_ratio = output_rms / input_rms;
        let gain_db = 20.0 * gain_ratio.log10();

        println!("{:9.1} | {:9.2}", freq, gain_db);
    }
}
