use std::f64::consts::PI;

use cmsis_interface::Q15;

const TABLE_SIZE: usize = 256;

fn main() {
    eprintln!("Generating sine wavetable:");
    eprintln!("  TABLE_SIZE: {}", TABLE_SIZE);
    eprintln!();

    println!("use cmsis_interface::Q15;");
    println!();
    print!("pub static SINE_WAVETABLE: [Q15; {}] = [", TABLE_SIZE);

    let mut samples = Vec::new();

    for i in 0..TABLE_SIZE {
        // Calculate sine value: sin(2π * i / TABLE_SIZE)
        let phase = 2.0 * PI * (i as f64) / (TABLE_SIZE as f64);
        let value = phase.sin();

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
        TABLE_SIZE / 4,
        samples[TABLE_SIZE / 4]
    );
    eprintln!(
        "  Sample at 180° (i={}): {} (expected: ~0.0)",
        TABLE_SIZE / 2,
        samples[TABLE_SIZE / 2]
    );
    eprintln!(
        "  Sample at 270° (i={}): {} (expected: ~-1.0)",
        TABLE_SIZE * 3 / 4,
        samples[TABLE_SIZE * 3 / 4]
    );
}
