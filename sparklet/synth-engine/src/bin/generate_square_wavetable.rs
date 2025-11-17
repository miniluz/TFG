use std::env;

use synth_engine::Q15;

fn main() {
    let args: Vec<String> = env::args().collect();

    let wavetable_size: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(256);

    eprintln!("Generating square wavetable:");
    eprintln!("  WAVETABLE_SIZE: {}", wavetable_size);
    eprintln!();

    print!("[");

    let mut samples = Vec::new();

    for i in 0..wavetable_size {
        // Calculate square value: 1 for first half, -1 for second half
        let value: f64 = if i < wavetable_size / 2 { 1.0 } else { -1.0 };

        // Convert to Q15 fixed-point format (1 sign bit, 15 fractional bits)
        // Range: [-1.0, 1.0) maps to [-32768, 32767]
        let fixed_value = (value * 32768.0).round() as i16;

        samples.push(Q15::from_bits(fixed_value));

        print!("Q15::from_bits({:#06x}_u16 as i16),", fixed_value as u16);
    }

    println!("]");

    eprintln!();
    eprintln!("Sanity checks:");
    eprintln!("  Sample at start (i=0): {} (expected: 1.0)", samples[0]);
    eprintln!(
        "  Sample at quarter (i={}): {} (expected: 1.0)",
        wavetable_size / 4,
        samples[wavetable_size / 4]
    );
    eprintln!(
        "  Sample at middle (i={}): {} (expected: -1.0)",
        wavetable_size / 2,
        samples[wavetable_size / 2]
    );
    eprintln!(
        "  Sample at 3/4 (i={}): {} (expected: -1.0)",
        wavetable_size * 3 / 4,
        samples[wavetable_size * 3 / 4]
    );
}
