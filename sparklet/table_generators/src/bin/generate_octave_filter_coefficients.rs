use std::env;

use fixed::types::I1F15 as Q15;
use std::f64::consts::SQRT_2;
use std::process::Command;

const SAMPLE_RATE: f64 = 48000.0;
const BAND_AMOUNT: usize = 6;

enum FilterType {
    LowPass(f64),
    BandPass(f64, f64),
    HighPass(f64),
}

struct FilterCoefficients {
    filter_type: FilterType,
    f64_coeffs: [f64; 5],
    q15_coeffs: [Q15; 6], // CMSIS format: [b0, 0, b1, b2, a1, a2]
    max_coeff: f64,
    saturated: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let coeff_shift: i8 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    assert!(coeff_shift >= 0);

    eprintln!("Generating octave filter coefficients for:");
    eprintln!("  SAMPLE_RATE: {} Hz", SAMPLE_RATE);
    eprintln!("  COEFF_SHIFT: {} bits", coeff_shift);

    let center_freqs = [250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0];

    let bands: Vec<FilterType> = center_freqs
        .iter()
        .enumerate()
        .map(|(idx, &freq)| {
            if idx == 0 {
                FilterType::LowPass(freq)
            } else if idx == center_freqs.len() - 1 {
                FilterType::HighPass(freq)
            } else {
                FilterType::BandPass(freq / SQRT_2, freq * SQRT_2)
            }
        })
        .collect();

    let mut filter_coeffs = Vec::new();

    for filter in bands.iter() {
        let f64_coeffs = get_octave_filter_coefficients(filter);
        let (q15_coeffs, max_coeff, saturated) = convert_to_q15(&f64_coeffs, coeff_shift);

        filter_coeffs.push(FilterCoefficients {
            filter_type: match filter {
                FilterType::LowPass(f) => FilterType::LowPass(*f),
                FilterType::BandPass(l, u) => FilterType::BandPass(*l, *u),
                FilterType::HighPass(f) => FilterType::HighPass(*f),
            },
            f64_coeffs,
            q15_coeffs,
            max_coeff,
            saturated,
        });
    }

    let contents = generate_output(&filter_coeffs, coeff_shift);
    println!("{}", contents);

    eprintln!();
    eprintln!("Sanity check:");
    for (band_idx, fc) in filter_coeffs.iter().enumerate() {
        let filter_name = match &fc.filter_type {
            FilterType::LowPass(freq) => format!("Low-pass at {:.1} Hz", freq),
            FilterType::BandPass(lower, upper) => {
                format!("Bandpass {:.1}-{:.1} Hz", lower, upper)
            }
            FilterType::HighPass(freq) => format!("High-pass at {:.1} Hz", freq),
        };

        eprintln!("  Band {} ({}):", band_idx, filter_name);
        eprintln!(
            "    Float coeffs: b0={:.6}, b1={:.6}, b2={:.6}, a1={:.6}, a2={:.6}",
            fc.f64_coeffs[0],
            fc.f64_coeffs[1],
            fc.f64_coeffs[2],
            fc.f64_coeffs[3],
            fc.f64_coeffs[4]
        );
        eprintln!(
            "    Q15 coeffs (CMSIS): [{}, {}, {}, {}, {}, {}]",
            fc.q15_coeffs[0].to_bits(),
            fc.q15_coeffs[1].to_bits(),
            fc.q15_coeffs[2].to_bits(),
            fc.q15_coeffs[3].to_bits(),
            fc.q15_coeffs[4].to_bits(),
            fc.q15_coeffs[5].to_bits()
        );
        eprintln!(
            "    Max coefficient magnitude: {:.6}{}",
            fc.max_coeff,
            if fc.saturated { " (SATURATED)" } else { "" }
        );
    }
}

/// Call octave to generate Butterworth filter coefficients
fn get_octave_filter_coefficients(filter: &FilterType) -> [f64; 5] {
    let nyquist = SAMPLE_RATE / 2.0;

    let octave_cmd = match filter {
        FilterType::LowPass(freq) => {
            let normalized = freq / nyquist;
            format!(
                "pkg load signal; [b,a] = butter(1, {}, 'low'); disp([b,a])",
                normalized
            )
        }
        FilterType::BandPass(lower_freq, upper_freq) => {
            let normalized_lower = lower_freq / nyquist;
            let normalized_upper = upper_freq / nyquist;
            format!(
                "pkg load signal; [b,a] = butter(1, [{},{}], 'bandpass'); disp([b,a])",
                normalized_lower, normalized_upper
            )
        }
        FilterType::HighPass(freq) => {
            let normalized = freq / nyquist;
            format!(
                "pkg load signal; [b,a] = butter(1, {}, 'high'); disp([b,a])",
                normalized
            )
        }
    };

    let output = Command::new("octave")
        .args(["-q", "--eval", &octave_cmd])
        .output()
        .expect("Failed to execute octave");

    if !output.status.success() {
        eprintln!("Octave stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Octave command failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse octave output - extract all floating point numbers
    let coeffs: Vec<f64> = stdout
        .split_whitespace()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();

    match filter {
        FilterType::LowPass(_) | FilterType::HighPass(_) => {
            // Low-pass and high-pass filters return 4 coefficients: [b0, b1, a0, a1]
            if coeffs.len() != 4 {
                panic!(
                    "Expected 4 coefficients from octave for low/high-pass, got {}: {:?}",
                    coeffs.len(),
                    coeffs
                );
            }
            // Return [b0, b1, 0, -a1, 0]
            // We discard a0 (which is 1.0) and set b2=0, a2=0
            [
                coeffs[0],  // b0
                coeffs[1],  // b1
                0.0,        // b2 = 0
                -coeffs[3], // -a1
                0.0,        // a2 = 0
            ]
        }
        FilterType::BandPass(_, _) => {
            // Bandpass filters return 6 coefficients: [b0, b1, b2, a0, a1, a2]
            if coeffs.len() != 6 {
                panic!(
                    "Expected 6 coefficients from octave for bandpass, got {}: {:?}",
                    coeffs.len(),
                    coeffs
                );
            }
            // Octave returns: [b0, b1, b2, a0, a1, a2]
            // We discard a0 (which is 1.0) and return [b0, b1, b2, a1, a2]
            // Note: Octave's convention is y[n] = b[0]*x[n] + ... - a[1]*y[n-1] - a[2]*y[n-2]
            // So we need to negate the a coefficients
            [
                coeffs[0],  // b0
                coeffs[1],  // b1
                coeffs[2],  // b2
                -coeffs[4], // -a1
                -coeffs[5], // -a2
            ]
        }
    }
}

fn convert_to_q15(coeffs: &[f64; 5], coeff_shift: i8) -> ([Q15; 6], f64, bool) {
    let b0 = coeffs[0];
    let b1 = coeffs[1];
    let b2 = coeffs[2];
    let a1 = coeffs[3];
    let a2 = coeffs[4];

    // Pre-shift coefficients right to prevent overflow during fixed-point computation
    let shift_factor = 2.0_f64.powi(coeff_shift as i32);
    let b0_shifted = b0 / shift_factor;
    let b1_shifted = b1 / shift_factor;
    let b2_shifted = b2 / shift_factor;
    let a1_shifted = a1 / shift_factor;
    let a2_shifted = a2 / shift_factor;

    // Check if coefficients are in valid range for Q15
    let max_coeff = b0_shifted
        .abs()
        .max(b1_shifted.abs())
        .max(b2_shifted.abs())
        .max(a1_shifted.abs())
        .max(a2_shifted.abs());

    let saturated = max_coeff >= 1.0;

    // Convert shifted coefficients to Q15 with saturation in CMSIS format
    // CMSIS format: [b0, 0, b1, b2, a1, a2] (zero padding for SIMD)
    let q15_coeffs = [
        Q15::saturating_from_num(b0_shifted),
        Q15::ZERO, // padding for SIMD
        Q15::saturating_from_num(b1_shifted),
        Q15::saturating_from_num(b2_shifted),
        Q15::saturating_from_num(a1_shifted),
        Q15::saturating_from_num(a2_shifted),
    ];

    (q15_coeffs, max_coeff, saturated)
}

fn generate_output(filter_coeffs: &[FilterCoefficients], coeff_shift: i8) -> String {
    let mut coeffs_array = String::new();

    for (band_idx, fc) in filter_coeffs.iter().enumerate() {
        let comment = match &fc.filter_type {
            FilterType::LowPass(freq) => format!("Band {} (Low-pass at {:.1} Hz)", band_idx, freq),
            FilterType::BandPass(lower, upper) => {
                format!("Band {} (Bandpass {:.1}-{:.1} Hz)", band_idx, lower, upper)
            }
            FilterType::HighPass(freq) => {
                format!("Band {} (High-pass at {:.1} Hz)", band_idx, freq)
            }
        };

        coeffs_array.push_str(&format!("    // {}\n", comment));
        coeffs_array.push_str(&format!("    {},\n", format_coeffs(&fc.q15_coeffs)));
    }

    format!(
        "//! Generated octave filter coefficients
//! DO NOT EDIT - Generated by generate_octave_filter_coefficients.rs

use cmsis_interface::Q15;

/// Post-shift value for biquad cascade filter
/// Coefficients are pre-shifted right by {} bits, so postShift = {}
pub const OCTAVE_FILTER_POST_SHIFT: i8 = {};

/// Octave filter coefficients for {} bands
/// Band 0: Low-pass filter
/// Bands 1-4: Bandpass filters (1st order Butterworth)
/// Band 5: High-pass filter
/// Coefficient format per stage: [b0, 0, b1, b2, a1, a2] (CMSIS format with zero padding)
/// Coefficients are pre-shifted right by {} bits to prevent overflow
pub static OCTAVE_FILTER_COEFFS: [[Q15; 6]; {}] = [
{}];
",
        coeff_shift, coeff_shift, coeff_shift, BAND_AMOUNT, coeff_shift, BAND_AMOUNT, coeffs_array
    )
}

fn format_coeffs(coeffs: &[Q15; 6]) -> String {
    format!(
        "[Q15::from_bits({}), Q15::from_bits({}), Q15::from_bits({}), Q15::from_bits({}), Q15::from_bits({}), Q15::from_bits({})]",
        coeffs[0].to_bits(),
        coeffs[1].to_bits(),
        coeffs[2].to_bits(),
        coeffs[3].to_bits(),
        coeffs[4].to_bits(),
        coeffs[5].to_bits(),
    )
}
