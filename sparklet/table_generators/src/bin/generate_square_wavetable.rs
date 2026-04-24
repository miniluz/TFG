use cmsis_interface::Q15;

const TABLE_SIZE: usize = 256;

fn main() {
    eprintln!("Generating square wavetable:");
    eprintln!("  TABLE_SIZE: {}", TABLE_SIZE);
    eprintln!();

    println!("use cmsis_interface::Q15;");
    println!();
    print!("pub static SQUARE_WAVETABLE: [Q15; {}] = [", TABLE_SIZE);

    let mut samples = Vec::new();

    for i in 0..TABLE_SIZE {
        // Calculate square value: 1 for first half, -1 for second half
        let value: f64 = if i < TABLE_SIZE / 2 { 1.0 } else { -1.0 };

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
    eprintln!("  Sample at start (i=0): {} (expected: 1.0)", samples[0]);
    eprintln!(
        "  Sample at quarter (i={}): {} (expected: 1.0)",
        TABLE_SIZE / 4,
        samples[TABLE_SIZE / 4]
    );
    eprintln!(
        "  Sample at middle (i={}): {} (expected: -1.0)",
        TABLE_SIZE / 2,
        samples[TABLE_SIZE / 2]
    );
    eprintln!(
        "  Sample at 3/4 (i={}): {} (expected: -1.0)",
        TABLE_SIZE * 3 / 4,
        samples[TABLE_SIZE * 3 / 4]
    );
}
