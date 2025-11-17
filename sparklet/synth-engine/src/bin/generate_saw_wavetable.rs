use std::env;

use synth_engine::Q15;

fn main() {
    let args: Vec<String> = env::args().collect();

    let wavetable_size: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(256);

    eprintln!("Generating sawtooth wavetable:");
    eprintln!("  WAVETABLE_SIZE: {}", wavetable_size);
    eprintln!();

    print!("[");

    let mut samples = Vec::new();

    for i in 0..wavetable_size {
        // Calculate sawtooth value: rises from -1 to just below 1
        // value = -1 + 2 * i / wavetable_size
        let value = -1.0 + 2.0 * (i as f64) / (wavetable_size as f64);

        // Convert to Q15 fixed-point format (1 sign bit, 15 fractional bits)
        // Range: [-1.0, 1.0) maps to [-32768, 32767]
        let fixed_value = (value * 32768.0).round() as i16;

        samples.push(Q15::from_bits(fixed_value));

        print!("Q15::from_bits({:#06x}_u16 as i16),", fixed_value as u16);
    }

    println!("]");

    eprintln!();
    eprintln!("Sanity checks:");
    eprintln!("  Sample at start (i=0): {} (expected: -1.0)", samples[0]);
    eprintln!(
        "  Sample at middle (i={}): {} (expected: ~0.0)",
        wavetable_size / 2,
        samples[wavetable_size / 2]
    );
    eprintln!(
        "  Sample at end (i={}): {} (expected: ~1.0)",
        wavetable_size - 1,
        samples[wavetable_size - 1]
    );
}
