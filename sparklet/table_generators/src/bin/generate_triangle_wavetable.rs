use std::env;

use cmsis_interface::Q15;

fn main() {
    let args: Vec<String> = env::args().collect();

    let wavetable_size: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(256);

    eprintln!("Generating triangle wavetable:");
    eprintln!("  WAVETABLE_SIZE: {}", wavetable_size);
    eprintln!();

    println!("use cmsis_interface::Q15;");
    println!();
    print!(
        "pub static TRIANGLE_WAVETABLE: [Q15; {}] = [",
        wavetable_size
    );

    let mut samples = Vec::new();

    for i in 0..wavetable_size {
        // Calculate triangle value
        // Starts at 0, rises to 1 at quarter point, falls to -1 at 3/4 point, returns to 0
        let phase = (i as f64) / (wavetable_size as f64);

        let value = if phase < 0.25 {
            4.0 * phase
        } else if phase < 0.75 {
            2.0 - 4.0 * phase
        } else {
            -4.0 + 4.0 * phase
        };

        // Convert to Q15 fixed-point format (1 sign bit, 15 fractional bits)
        // Range: [-1.0, 1.0) maps to [-32768, 32767]
        let fixed_value = (value * 32768.0).round() as i16;

        samples.push(Q15::from_bits(fixed_value));

        println!();
        print!(
            "    Q15::from_bits({:#06x}_u16 as i16),",
            fixed_value as u16
        );
    }

    println!();
    println!("];");

    eprintln!();
    eprintln!("Sanity checks:");
    eprintln!("  Sample at 0° (i=0): {} (expected: 0.0)", samples[0]);
    eprintln!(
        "  Sample at 90° (i={}): {} (expected: ~1.0)",
        wavetable_size / 4,
        samples[wavetable_size / 4]
    );
    eprintln!(
        "  Sample at 180° (i={}): {} (expected: ~0.0)",
        wavetable_size / 2,
        samples[wavetable_size / 2]
    );
    eprintln!(
        "  Sample at 270° (i={}): {} (expected: ~-1.0)",
        wavetable_size * 3 / 4,
        samples[wavetable_size * 3 / 4]
    );
}
