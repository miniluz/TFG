//! Native utilities for ADSR envelope tables.
//!
//! This module provides utilities for calculating exponential envelope curves
//! and generating lookup tables. These functions use floating-point math and
//! are only available on host platforms (not embedded targets).
//!
//! The core algorithm uses the recursive formula:
//! ```text
//! output = base + output * coefficient
//! ```
//!
//! This module is conditionally compiled and only available when not targeting
//! embedded platforms (target_os != "none").

// This module requires std for floating-point math and string formatting
extern crate std;
use std::prelude::v1::*;

use fixed::{
    traits::{Fixed, ToFixed},
    types::I1F31,
};

use crate::adsr::{ADSRStage, BaseAndCoefficient};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct DecayConfig {
    /// Rate: number of samples for the curve
    pub rate: f64,
    /// Target ratio: controls the exponential curve shape, must be more than 0 and the higher it
    /// is the more linear the curve gets.
    pub target_ratio: f64,
    /// Initial value of the envelope
    pub initial: f64,
    /// Target value of the envelope
    pub target: f64,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct TimeConfig {
    /// Rate: maximum index value (typically table size - 1)
    pub rate: f64,
    /// Ratio: controls exponential vs linear time mapping. Goes from -infinity to infinity.
    pub ratio: f64,
    /// Initial time value (in seconds)
    pub initial: f64,
    /// Target time value (in seconds)
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
    // coefficient = e^(ln(target_ratio / (1 + target_ratio)) / rate)
    let coefficient = f64::exp(f64::ln(target_ratio / (1. + target_ratio)) / rate);

    // Calculate final value: initial + (target - initial) * (1 + target_ratio)
    // base = final * (1 - coefficient)
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
    // time = initial + (target - initial) * (e^(ratio + (ln(e^ratio + 1) - ratio) * x/rate) - e^ratio)
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
        output = ADSRStage::decay(output, BaseAndCoefficient { base, coefficient });
        if output == I1F31::MAX {
            return (iteration + 1, I1F31::MAX);
        } else if output < I1F31::ZERO {
            return (iteration + 1, I1F31::ZERO);
        }
    }

    (iterations, output)
}
