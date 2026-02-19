use fixed::{traits::ToFixed, types::I1F31};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct BaseAndCoefficient {
    pub base: I1F31,
    pub coefficient: I1F31,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct DecayConfig {
    pub rate: f64,
    pub target_ratio: f64,
    pub initial: f64,
    pub target: f64,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct TimeConfig {
    pub rate: f64,
    pub ratio: f64,
    pub initial: f64,
    pub target: f64,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct ParamConfig {
    pub target_ratio: f64,
    pub initial: f64,
    pub target: f64,
    pub time_config: TimeConfig,
}

pub fn get_base_and_coefficient<const SAMPLE_RATE: u32>(
    DecayConfig {
        rate,
        target_ratio,
        initial,
        target,
    }: DecayConfig,
) -> BaseAndCoefficient {
    let coefficient = f64::exp(f64::ln(target_ratio / (1. + target_ratio)) / rate);
    let r#final = initial + (target - initial) * (1. + target_ratio);
    let base = r#final * (1. - coefficient);

    let base = base.to_fixed();
    let coefficient = coefficient.to_fixed();

    BaseAndCoefficient { base, coefficient }
}

pub fn get_time_for_index<const SAMPLE_RATE: u32>(
    index: usize,
    TimeConfig {
        rate,
        ratio,
        initial,
        target,
    }: TimeConfig,
) -> f64 {
    initial
        + (target - initial)
            * (f64::exp(ratio + (f64::ln(f64::exp(ratio) + 1.) - ratio) * (index as f64) / rate)
                - f64::exp(ratio))
}

pub fn get_base_and_coefficient_for_index<const SAMPLE_RATE: u32>(
    index: usize,
    ParamConfig {
        target_ratio,
        initial,
        target,
        time_config,
    }: ParamConfig,
) -> BaseAndCoefficient {
    let time = get_time_for_index::<SAMPLE_RATE>(index, time_config);
    let rate = time * SAMPLE_RATE as f64;

    get_base_and_coefficient::<SAMPLE_RATE>(DecayConfig {
        rate,
        target_ratio,
        initial,
        target,
    })
}

pub fn iterate_envelope(
    base: I1F31,
    coefficient: I1F31,
    initial: I1F31,
    iterations: usize,
) -> (usize, I1F31) {
    let mut output = initial;

    for iteration in 1..iterations {
        output = output.saturating_mul_add(coefficient, base);
        if output == I1F31::MAX {
            return (iteration + 1, I1F31::MAX);
        } else if output < I1F31::ZERO {
            return (iteration + 1, I1F31::ZERO);
        }
    }

    (iterations, output)
}
