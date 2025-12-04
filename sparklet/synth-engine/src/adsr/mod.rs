use cmsis_interface::Q15;
use defmt::Format;
use fixed::types::I1F31;

use crate::adsr::{
    config_table::{
        RISE_BASE_COEFFICIENT_TABLE, FALL_BASE_COEFFICIENT_TABLE,
    },
    db_linear_amplitude_table::DB_LINEAR_AMPLITUDE_TABLE,
};
use crate::capacitor::{Capacitor, CapacitorStatus};

pub mod config_table;
pub mod db_linear_amplitude_table;
#[cfg(not(target_os = "none"))]
pub mod native_utils;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaseAndCoefficient {
    pub base: I1F31,
    pub coefficient: I1F31,
}

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ADSRStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
    QuickRelease,
}

impl ADSRStage {
    pub(crate) fn play(&mut self) {
        *self = Self::Attack;
    }

    pub(crate) fn stop_playing(&mut self) {
        match *self {
            Self::Idle => (),
            _ => {
                *self = ADSRStage::Release;
            }
        }
    }

    pub(crate) fn quick_release(&mut self) {
        match *self {
            Self::Idle => (),
            _ => {
                *self = ADSRStage::QuickRelease;
            }
        }
    }

    pub(crate) fn progress(&mut self, capacitor: &mut Capacitor, adsr_config: &ADSRConfig) -> I1F31 {
        match *self {
            Self::Idle => I1F31::ZERO,
            Self::Attack => {
                capacitor.set_target(adsr_config.velocity_amplitude);
                let status = capacitor.step();
                if status == CapacitorStatus::ReachedTarget {
                    *self = Self::Decay;
                }
                capacitor.get_level()
            }
            Self::Decay => {
                let decay_target = adsr_config.sustain_level
                    .saturating_mul(adsr_config.velocity_amplitude);
                capacitor.set_target(decay_target);
                let status = capacitor.step();
                if status == CapacitorStatus::ReachedTarget {
                    *self = Self::Sustain;
                }
                capacitor.get_level()
            }
            Self::Sustain => capacitor.get_level(),
            Self::Release => {
                capacitor.set_target(I1F31::ZERO);
                let status = capacitor.step();
                if status == CapacitorStatus::ReachedTarget {
                    *self = Self::Idle;
                }
                capacitor.get_level()
            }
            Self::QuickRelease => {
                capacitor.quick_discharge();
                let status = capacitor.step();
                if status == CapacitorStatus::ReachedTarget {
                    *self = Self::Idle;
                }
                capacitor.get_level()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ADSRConfig {
    sustain_level: I1F31,
    velocity_amplitude: I1F31,
    rise_base_and_coefficient: BaseAndCoefficient,
    fall_base_and_coefficient: BaseAndCoefficient,
}

impl ADSRConfig {
    pub(crate) fn new(sustain_config: u8, attack_config: u8, decay_release_config: u8, velocity: u8) -> Self {
        Self {
            sustain_level: DB_LINEAR_AMPLITUDE_TABLE[sustain_config as usize],
            velocity_amplitude: DB_LINEAR_AMPLITUDE_TABLE[(velocity * 2) as usize],
            rise_base_and_coefficient: RISE_BASE_COEFFICIENT_TABLE[attack_config as usize],
            fall_base_and_coefficient: FALL_BASE_COEFFICIENT_TABLE
                [decay_release_config as usize],
        }
    }

    pub(crate) fn set_velocity(&mut self, velocity: u8) {
        self.velocity_amplitude = DB_LINEAR_AMPLITUDE_TABLE[(velocity * 2) as usize];
    }

    pub(crate) fn set_sustain(&mut self, sustain_config: u8) {
        self.sustain_level = DB_LINEAR_AMPLITUDE_TABLE[sustain_config as usize];
    }

    pub(crate) fn set_rise(&mut self, attack_config: u8) {
        self.rise_base_and_coefficient = RISE_BASE_COEFFICIENT_TABLE[attack_config as usize];
    }

    pub(crate) fn set_fall(&mut self, decay_release_config: u8) {
        self.fall_base_and_coefficient =
            FALL_BASE_COEFFICIENT_TABLE[decay_release_config as usize];
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ADSR {
    pub(crate) stage: ADSRStage,
    pub(crate) config: ADSRConfig,
    pub(crate) capacitor: Capacitor,
}

impl Format for ADSR {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ADSR {{ stage: {} }}", self.stage)
    }
}

impl ADSR {
    pub fn new(sustain_config: u8, attack_config: u8, decay_release_config: u8, velocity: u8) -> ADSR {
        let config = ADSRConfig::new(sustain_config, attack_config, decay_release_config, velocity);
        ADSR {
            stage: ADSRStage::Idle,
            capacitor: Capacitor::new(config.rise_base_and_coefficient, config.fall_base_and_coefficient),
            config,
        }
    }

    pub fn set_velocity(&mut self, velocity: u8) {
        self.config.set_velocity(velocity);
    }

    pub fn play(&mut self, velocity: u8) {
        self.capacitor.set_level(I1F31::ZERO);
        self.config.set_velocity(velocity);
        self.stage.play()
    }

    pub fn retrigger(&mut self, velocity: u8) {
        // Don't reset capacitor level on retrigger - continue from current
        self.config.set_velocity(velocity);
        self.stage.play()
    }

    pub fn stop_playing(&mut self) {
        self.stage.stop_playing()
    }

    pub fn quick_release(&mut self) {
        self.stage.quick_release()
    }

    pub fn get_samples<const LEN: usize>(&mut self, buffer: &mut [Q15; LEN]) {
        for elem in buffer.iter_mut() {
            let output = self.stage.progress(&mut self.capacitor, &self.config);
            *elem = fixed::traits::LossyInto::lossy_into(output);
        }
    }

    pub fn set_sustain(&mut self, sustain_config: u8) {
        self.config.set_sustain(sustain_config);
    }

    pub fn set_attack(&mut self, attack_config: u8) {
        self.config.set_rise(attack_config);
        self.capacitor.set_rise_coeff(self.config.rise_base_and_coefficient);
    }

    pub fn set_decay_release(&mut self, decay_release_config: u8) {
        self.config.set_fall(decay_release_config);
        self.capacitor.set_fall_coeff(self.config.fall_base_and_coefficient);
    }

    pub fn is_idle(&self) -> bool {
        self.stage == ADSRStage::Idle
    }

    pub fn is_in_release(&self) -> bool {
        self.stage == ADSRStage::Release
    }

    pub fn is_in_quick_release(&self) -> bool {
        self.stage == ADSRStage::QuickRelease
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper functions
    fn create_test_adsr() -> ADSR {
        ADSR::new(200, 50, 100, 100) // Medium sustain, fast attack/decay
    }

    fn advance_to_stage(adsr: &mut ADSR, target_stage: ADSRStage, max_iterations: usize) -> bool {
        let mut buffer = [Q15::ZERO; 1];
        for _ in 0..max_iterations {
            adsr.get_samples(&mut buffer);
            if adsr.stage == target_stage {
                return true;
            }
        }
        false // Failed to reach target stage
    }

    fn get_envelope_level(adsr: &ADSR) -> I1F31 {
        adsr.capacitor.get_level()
    }

    // 1. Stage Transition Tests

    #[test]
    fn test_attack_to_decay_transition() {
        let mut adsr = ADSR::new(200, 10, 100, 100); // Fast attack
        adsr.play(100);

        assert_eq!(adsr.stage, ADSRStage::Attack, "Should start in Attack stage");

        let mut prev_level = I1F31::ZERO;
        let mut reached_decay = false;

        for _ in 0..10000 {
            let mut buffer = [Q15::ZERO; 1];
            adsr.get_samples(&mut buffer);
            let current_level = get_envelope_level(&adsr);

            if adsr.stage == ADSRStage::Decay {
                reached_decay = true;
                break;
            }

            assert!(
                current_level >= prev_level,
                "Attack envelope should increase monotonically"
            );
            prev_level = current_level;
        }

        assert!(reached_decay, "Should transition from Attack to Decay");
    }

    #[test]
    fn test_decay_to_sustain_transition() {
        let mut adsr = ADSR::new(200, 10, 10, 100); // Fast attack and decay
        adsr.play(100);

        // Advance through attack
        assert!(
            advance_to_stage(&mut adsr, ADSRStage::Decay, 10000),
            "Should reach Decay stage"
        );

        let mut prev_level = get_envelope_level(&adsr);
        let mut reached_sustain = false;

        for _ in 0..10000 {
            let mut buffer = [Q15::ZERO; 1];
            adsr.get_samples(&mut buffer);
            let current_level = get_envelope_level(&adsr);

            if adsr.stage == ADSRStage::Sustain {
                reached_sustain = true;
                break;
            }

            assert!(
                current_level <= prev_level,
                "Decay envelope should decrease monotonically"
            );
            prev_level = current_level;
        }

        assert!(reached_sustain, "Should transition from Decay to Sustain");
    }

    #[test]
    fn test_sustain_holds_steady() {
        let mut adsr = create_test_adsr();
        adsr.play(100);

        // Advance to sustain
        assert!(
            advance_to_stage(&mut adsr, ADSRStage::Sustain, 10000),
            "Should reach Sustain stage"
        );

        let sustain_level = get_envelope_level(&adsr);

        // Get many samples and verify level stays constant
        for _ in 0..1000 {
            let mut buffer = [Q15::ZERO; 1];
            adsr.get_samples(&mut buffer);
            assert_eq!(
                get_envelope_level(&adsr),
                sustain_level,
                "Sustain level should remain constant"
            );
            assert_eq!(adsr.stage, ADSRStage::Sustain, "Should remain in Sustain");
        }
    }

    #[test]
    fn test_release_to_idle_transition() {
        let mut adsr = create_test_adsr();
        adsr.play(100);

        // Advance to sustain
        assert!(
            advance_to_stage(&mut adsr, ADSRStage::Sustain, 10000),
            "Should reach Sustain stage"
        );

        adsr.stop_playing();
        assert_eq!(adsr.stage, ADSRStage::Release, "Should enter Release stage");

        // Advance through release
        assert!(
            advance_to_stage(&mut adsr, ADSRStage::Idle, 10000),
            "Should reach Idle stage"
        );

        assert_eq!(
            get_envelope_level(&adsr),
            I1F31::ZERO,
            "Should have zero level when idle"
        );
    }

    #[test]
    fn test_quick_release_faster_than_normal() {
        // Create two ADSRs with slow release but fast attack/decay
        let mut adsr_normal = ADSR::new(200, 10, 200, 100); // Fast attack, slow release (index 200)
        let mut adsr_quick = ADSR::new(200, 10, 200, 100);

        // Advance both to sustain
        adsr_normal.play(100);
        adsr_quick.play(100);

        assert!(
            advance_to_stage(&mut adsr_normal, ADSRStage::Sustain, 50000),
            "Normal ADSR should reach Sustain"
        );
        assert!(
            advance_to_stage(&mut adsr_quick, ADSRStage::Sustain, 50000),
            "Quick ADSR should reach Sustain"
        );

        // Trigger releases
        adsr_normal.stop_playing();
        adsr_quick.quick_release();

        // Count iterations to idle
        let mut normal_iterations = 0;
        let mut quick_iterations = 0;

        let mut buffer = [Q15::ZERO; 1];

        while !adsr_normal.is_idle() && normal_iterations < 100000 {
            adsr_normal.get_samples(&mut buffer);
            normal_iterations += 1;
        }

        while !adsr_quick.is_idle() && quick_iterations < 100000 {
            adsr_quick.get_samples(&mut buffer);
            quick_iterations += 1;
        }

        assert!(
            quick_iterations < normal_iterations,
            "Quick release ({} iterations) should be faster than normal release ({} iterations)",
            quick_iterations,
            normal_iterations
        );
    }

    // 2. Envelope Behavior Tests

    #[test]
    fn test_attack_envelope_increases_monotonically() {
        let mut adsr = ADSR::new(200, 50, 100, 100); // Medium attack speed
        adsr.play(100);

        let mut levels = Vec::new();
        let mut buffer = [Q15::ZERO; 1];

        // Collect samples during attack phase
        for _ in 0..1000 {
            adsr.get_samples(&mut buffer);
            levels.push(get_envelope_level(&adsr));

            if adsr.stage != ADSRStage::Attack {
                break;
            }
        }

        // Verify monotonically increasing
        for i in 1..levels.len() {
            assert!(
                levels[i] >= levels[i - 1],
                "Attack envelope should increase monotonically: level[{}]={:?} < level[{}]={:?}",
                i,
                levels[i],
                i - 1,
                levels[i - 1]
            );
        }

        assert!(!levels.is_empty(), "Should have collected some attack samples");
    }

    #[test]
    fn test_decay_envelope_decreases_monotonically() {
        let mut adsr = ADSR::new(100, 10, 100, 100); // Low sustain, fast attack
        adsr.play(100);

        // Advance to decay
        assert!(
            advance_to_stage(&mut adsr, ADSRStage::Decay, 10000),
            "Should reach Decay stage"
        );

        let mut levels = Vec::new();
        let mut buffer = [Q15::ZERO; 1];

        // Collect samples during decay phase
        for _ in 0..1000 {
            adsr.get_samples(&mut buffer);
            levels.push(get_envelope_level(&adsr));

            if adsr.stage != ADSRStage::Decay {
                break;
            }
        }

        // Verify monotonically decreasing
        for i in 1..levels.len() {
            assert!(
                levels[i] <= levels[i - 1],
                "Decay envelope should decrease monotonically"
            );
        }

        assert!(!levels.is_empty(), "Should have collected some decay samples");
    }

    #[test]
    fn test_release_decreases_to_zero() {
        let mut adsr = create_test_adsr();
        adsr.play(100);

        // Advance to sustain
        assert!(
            advance_to_stage(&mut adsr, ADSRStage::Sustain, 10000),
            "Should reach Sustain stage"
        );

        adsr.stop_playing();

        let mut levels = Vec::new();
        let mut buffer = [Q15::ZERO; 1];

        // Collect samples during release phase
        for _ in 0..10000 {
            adsr.get_samples(&mut buffer);
            levels.push(get_envelope_level(&adsr));

            if adsr.stage == ADSRStage::Idle {
                break;
            }
        }

        // Verify monotonically decreasing
        for i in 1..levels.len() {
            assert!(
                levels[i] <= levels[i - 1],
                "Release envelope should decrease monotonically"
            );
        }

        assert_eq!(adsr.stage, ADSRStage::Idle, "Should reach Idle stage");
        assert_eq!(
            get_envelope_level(&adsr),
            I1F31::ZERO,
            "Final level should be zero"
        );
    }

    #[test]
    fn test_idle_outputs_zero() {
        let mut adsr = create_test_adsr();
        // Don't call play()

        let mut buffer = [Q15::ZERO; 100];
        adsr.get_samples(&mut buffer);

        // Verify all samples are zero
        for sample in buffer.iter() {
            assert_eq!(*sample, Q15::ZERO, "Idle ADSR should output zero");
        }

        assert_eq!(adsr.stage, ADSRStage::Idle, "Should remain in Idle stage");
    }

    // 3. Velocity Scaling Tests

    #[test]
    fn test_velocity_affects_attack_peak() {
        let mut adsr_low = ADSR::new(200, 50, 100, 64); // Lower velocity
        let mut adsr_high = ADSR::new(200, 50, 100, 127); // Higher velocity

        adsr_low.play(64);
        adsr_high.play(127);

        // Advance both to decay (past attack peak)
        assert!(
            advance_to_stage(&mut adsr_low, ADSRStage::Decay, 10000),
            "Low velocity ADSR should reach Decay"
        );
        assert!(
            advance_to_stage(&mut adsr_high, ADSRStage::Decay, 10000),
            "High velocity ADSR should reach Decay"
        );

        let low_peak = get_envelope_level(&adsr_low);
        let high_peak = get_envelope_level(&adsr_high);

        assert!(
            high_peak > low_peak,
            "Higher velocity should produce higher attack peak: high={:?}, low={:?}",
            high_peak,
            low_peak
        );
    }

    #[test]
    fn test_velocity_affects_sustain_amplitude() {
        let mut adsr_low = ADSR::new(200, 10, 10, 64); // Faster attack and decay
        let mut adsr_high = ADSR::new(200, 10, 10, 127);

        adsr_low.play(64);
        adsr_high.play(127);

        // Advance both to sustain
        assert!(
            advance_to_stage(&mut adsr_low, ADSRStage::Sustain, 50000),
            "Low velocity ADSR should reach Sustain"
        );
        assert!(
            advance_to_stage(&mut adsr_high, ADSRStage::Sustain, 50000),
            "High velocity ADSR should reach Sustain"
        );

        let low_sustain = get_envelope_level(&adsr_low);
        let high_sustain = get_envelope_level(&adsr_high);

        assert!(
            high_sustain > low_sustain,
            "Higher velocity should produce higher sustain level: high={:?}, low={:?}",
            high_sustain,
            low_sustain
        );
    }

    #[test]
    fn test_velocity_zero_produces_zero() {
        let mut adsr = ADSR::new(200, 50, 100, 1); // Minimum velocity
        adsr.play(1);

        // Get samples through full envelope
        let mut buffer = [Q15::ZERO; 100];

        for _ in 0..100 {
            adsr.get_samples(&mut buffer);
        }

        // All samples should be very small (near zero)
        for sample in buffer.iter() {
            let abs_val = if *sample < Q15::ZERO {
                -sample.to_bits()
            } else {
                sample.to_bits()
            };
            assert!(
                abs_val < 1000, // Very small threshold
                "Minimum velocity should produce near-zero samples, got {:?}",
                sample
            );
        }
    }

    // 4. Sustain Level Tests

    #[test]
    fn test_sustain_level_affects_decay_target() {
        let mut adsr_high_sustain = ADSR::new(255, 50, 100, 100); // High sustain
        let mut adsr_low_sustain = ADSR::new(50, 50, 100, 100); // Low sustain

        adsr_high_sustain.play(100);
        adsr_low_sustain.play(100);

        // Advance both to sustain
        assert!(
            advance_to_stage(&mut adsr_high_sustain, ADSRStage::Sustain, 10000),
            "High sustain ADSR should reach Sustain"
        );
        assert!(
            advance_to_stage(&mut adsr_low_sustain, ADSRStage::Sustain, 10000),
            "Low sustain ADSR should reach Sustain"
        );

        let high_level = get_envelope_level(&adsr_high_sustain);
        let low_level = get_envelope_level(&adsr_low_sustain);

        assert!(
            high_level > low_level,
            "Higher sustain config should produce higher sustain level: high={:?}, low={:?}",
            high_level,
            low_level
        );
    }

    #[test]
    fn test_sustain_zero_decays_to_zero() {
        let mut adsr = ADSR::new(0, 50, 100, 100); // Zero sustain
        adsr.play(100);

        // Advance to sustain
        assert!(
            advance_to_stage(&mut adsr, ADSRStage::Sustain, 10000),
            "Should reach Sustain stage"
        );

        let sustain_level = get_envelope_level(&adsr);

        // Should be at or very near zero
        assert!(
            sustain_level < I1F31::from_num(0.01),
            "Zero sustain config should produce near-zero level, got {:?}",
            sustain_level
        );
    }

    // 5. Configuration Change Tests

    #[test]
    fn test_set_attack_while_playing() {
        let mut adsr = ADSR::new(200, 200, 100, 100); // Slow attack
        adsr.play(100);

        // Advance partway through attack
        for _ in 0..100 {
            let mut buffer = [Q15::ZERO; 1];
            adsr.get_samples(&mut buffer);
        }

        assert_eq!(adsr.stage, ADSRStage::Attack, "Should be in Attack stage");

        // Change to fast attack
        adsr.set_attack(10);

        // Verify it reaches decay (doesn't get stuck)
        assert!(
            advance_to_stage(&mut adsr, ADSRStage::Decay, 10000),
            "Should reach Decay after changing attack config"
        );
    }

    #[test]
    fn test_set_sustain_in_decay() {
        let mut adsr = ADSR::new(200, 10, 100, 100);
        adsr.play(100);

        // Advance to decay
        assert!(
            advance_to_stage(&mut adsr, ADSRStage::Decay, 10000),
            "Should reach Decay stage"
        );

        // Change sustain level
        adsr.set_sustain(100); // Lower sustain

        // Advance to sustain
        assert!(
            advance_to_stage(&mut adsr, ADSRStage::Sustain, 10000),
            "Should reach Sustain stage"
        );

        let final_level = get_envelope_level(&adsr);

        // Verify it reached the new sustain target (approximately)
        // Since sustain config 100 maps to a specific amplitude via the table
        let expected = db_linear_amplitude_table::DB_LINEAR_AMPLITUDE_TABLE[100];
        let velocity_amplitude = db_linear_amplitude_table::DB_LINEAR_AMPLITUDE_TABLE[200];
        let expected_sustain = expected.saturating_mul(velocity_amplitude);

        // Allow some tolerance due to rounding
        let diff = if final_level > expected_sustain {
            final_level - expected_sustain
        } else {
            expected_sustain - final_level
        };

        assert!(
            diff < I1F31::from_num(0.01),
            "Should reach new sustain target: expected {:?}, got {:?}",
            expected_sustain,
            final_level
        );
    }

    #[test]
    fn test_set_decay_release_while_in_release() {
        let mut adsr = ADSR::new(200, 10, 200, 100); // Fast attack, slow release
        adsr.play(100);

        // Advance to sustain
        assert!(
            advance_to_stage(&mut adsr, ADSRStage::Sustain, 50000),
            "Should reach Sustain stage"
        );

        adsr.stop_playing();
        assert_eq!(adsr.stage, ADSRStage::Release);

        // Let some slow release happen
        for _ in 0..100 {
            let mut buffer = [Q15::ZERO; 1];
            adsr.get_samples(&mut buffer);
        }

        // Change to fast release
        adsr.set_decay_release(10);

        // Count iterations to idle with fast release
        let mut iterations = 0;
        while !adsr.is_idle() && iterations < 10000 {
            let mut buffer = [Q15::ZERO; 1];
            adsr.get_samples(&mut buffer);
            iterations += 1;
        }

        assert!(adsr.is_idle(), "Should reach idle after changing to fast release");
        assert!(
            iterations < 5000,
            "Should reach idle relatively quickly with fast release, took {} iterations",
            iterations
        );
    }

    // 6. Retrigger Tests

    #[test]
    fn test_retrigger_vs_play() {
        let mut adsr = create_test_adsr();
        adsr.play(100);

        // Advance partway through attack
        for _ in 0..100 {
            let mut buffer = [Q15::ZERO; 1];
            adsr.get_samples(&mut buffer);
        }

        let level_before_retrigger = get_envelope_level(&adsr);
        assert!(
            level_before_retrigger > I1F31::ZERO,
            "Should have non-zero level before retrigger"
        );

        // Retrigger
        adsr.retrigger(100);
        assert_eq!(adsr.stage, ADSRStage::Attack, "Should be in Attack after retrigger");

        // Get one sample
        let mut buffer = [Q15::ZERO; 1];
        adsr.get_samples(&mut buffer);

        let level_after_retrigger = get_envelope_level(&adsr);

        // Level should continue from previous (not reset to zero)
        assert!(
            level_after_retrigger >= level_before_retrigger,
            "Retrigger should continue from current level: before={:?}, after={:?}",
            level_before_retrigger,
            level_after_retrigger
        );
    }

    #[test]
    fn test_play_resets_level() {
        let mut adsr = create_test_adsr();
        adsr.play(100);

        // Advance to sustain
        assert!(
            advance_to_stage(&mut adsr, ADSRStage::Sustain, 10000),
            "Should reach Sustain stage"
        );

        let sustain_level = get_envelope_level(&adsr);
        assert!(sustain_level > I1F31::ZERO, "Should have non-zero sustain level");

        // Play again
        adsr.play(100);

        // Get one sample
        let mut buffer = [Q15::ZERO; 1];
        adsr.get_samples(&mut buffer);

        let level_after_play = get_envelope_level(&adsr);

        // Level should be reset to near zero (fresh attack)
        assert!(
            level_after_play < I1F31::from_num(0.1),
            "Play should reset level to near zero, got {:?}",
            level_after_play
        );
    }

    #[test]
    fn test_retrigger_different_velocity_transitions_gradually() {
        // Test 1: Decreasing velocity (high to low)
        let mut adsr = create_test_adsr();
        adsr.play(127); // Start with high velocity

        // Advance partway through attack
        for _ in 0..100 {
            let mut buffer = [Q15::ZERO; 1];
            adsr.get_samples(&mut buffer);
        }

        let level_before_retrigger = get_envelope_level(&adsr);
        assert!(
            level_before_retrigger > I1F31::ZERO,
            "Should have non-zero level before retrigger"
        );

        // Retrigger with much lower velocity
        adsr.retrigger(64);
        assert_eq!(adsr.stage, ADSRStage::Attack, "Should be in Attack after retrigger");

        // Get a few samples and track the transition
        let mut buffer = [Q15::ZERO; 1];
        let mut levels = Vec::new();

        for _ in 0..10 {
            adsr.get_samples(&mut buffer);
            levels.push(get_envelope_level(&adsr));
        }

        // Verify the transition is gradual, not a sudden jump
        // The level should not jump down immediately to match the new velocity target
        // Instead, it should continue from the current level and gradually adjust

        // Check that the first sample after retrigger didn't jump down drastically
        let first_level_after = levels[0];
        let level_diff = if level_before_retrigger > first_level_after {
            level_before_retrigger - first_level_after
        } else {
            first_level_after - level_before_retrigger
        };

        // The change in one sample should be small (gradual transition)
        assert!(
            level_diff < I1F31::from_num(0.1),
            "Level should not jump suddenly on retrigger with decreasing velocity: before={:?}, after={:?}, diff={:?}",
            level_before_retrigger,
            first_level_after,
            level_diff
        );

        // Verify levels change gradually over multiple samples
        for i in 1..levels.len() {
            let step_diff = if levels[i] > levels[i-1] {
                levels[i] - levels[i-1]
            } else {
                levels[i-1] - levels[i]
            };

            assert!(
                step_diff < I1F31::from_num(0.1),
                "Each step should be gradual, not sudden jumps (decreasing): step {}->{}",
                i-1, i
            );
        }

        // Test 2: Increasing velocity (low to high)
        let mut adsr2 = create_test_adsr();
        adsr2.play(64); // Start with low velocity

        // Advance partway through attack
        for _ in 0..100 {
            let mut buffer = [Q15::ZERO; 1];
            adsr2.get_samples(&mut buffer);
        }

        let level_before_retrigger2 = get_envelope_level(&adsr2);
        assert!(
            level_before_retrigger2 > I1F31::ZERO,
            "Should have non-zero level before retrigger"
        );

        // Retrigger with much higher velocity
        adsr2.retrigger(127);
        assert_eq!(adsr2.stage, ADSRStage::Attack, "Should be in Attack after retrigger");

        // Get a few samples and track the transition
        let mut levels2 = Vec::new();

        for _ in 0..10 {
            adsr2.get_samples(&mut buffer);
            levels2.push(get_envelope_level(&adsr2));
        }

        // Check that the first sample after retrigger didn't jump up drastically
        let first_level_after2 = levels2[0];
        let level_diff2 = if first_level_after2 > level_before_retrigger2 {
            first_level_after2 - level_before_retrigger2
        } else {
            level_before_retrigger2 - first_level_after2
        };

        // The change in one sample should be small (gradual transition)
        assert!(
            level_diff2 < I1F31::from_num(0.1),
            "Level should not jump suddenly on retrigger with increasing velocity: before={:?}, after={:?}, diff={:?}",
            level_before_retrigger2,
            first_level_after2,
            level_diff2
        );

        // Verify levels change gradually over multiple samples
        for i in 1..levels2.len() {
            let step_diff = if levels2[i] > levels2[i-1] {
                levels2[i] - levels2[i-1]
            } else {
                levels2[i-1] - levels2[i]
            };

            assert!(
                step_diff < I1F31::from_num(0.1),
                "Each step should be gradual, not sudden jumps (increasing): step {}->{}",
                i-1, i
            );
        }
    }

    // 7. Edge Cases

    #[test]
    fn test_stop_playing_when_idle() {
        let mut adsr = create_test_adsr();
        // Don't call play

        assert!(adsr.is_idle(), "Should start idle");

        adsr.stop_playing();

        assert!(adsr.is_idle(), "Should remain idle after stop_playing");

        let mut buffer = [Q15::ZERO; 10];
        adsr.get_samples(&mut buffer);

        for sample in buffer.iter() {
            assert_eq!(*sample, Q15::ZERO, "Should output zero when idle");
        }
    }

    #[test]
    fn test_quick_release_when_idle() {
        let mut adsr = create_test_adsr();

        assert!(adsr.is_idle(), "Should start idle");

        adsr.quick_release();

        assert!(adsr.is_idle(), "Should remain idle after quick_release");
    }

    #[test]
    fn test_multiple_play_calls() {
        let mut adsr = create_test_adsr();
        adsr.play(100);

        // Advance partway through attack
        for _ in 0..100 {
            let mut buffer = [Q15::ZERO; 1];
            adsr.get_samples(&mut buffer);
        }

        let level_mid_attack = get_envelope_level(&adsr);
        assert!(level_mid_attack > I1F31::ZERO, "Should have progressed in attack");

        // Call play again
        adsr.play(100);

        assert_eq!(adsr.stage, ADSRStage::Attack, "Should be in Attack stage");

        // Get one sample
        let mut buffer = [Q15::ZERO; 1];
        adsr.get_samples(&mut buffer);

        let level_after_second_play = get_envelope_level(&adsr);

        // Should restart from zero (like fresh note)
        assert!(
            level_after_second_play < level_mid_attack,
            "Second play() should restart envelope: previous={:?}, new={:?}",
            level_mid_attack,
            level_after_second_play
        );
    }
}
