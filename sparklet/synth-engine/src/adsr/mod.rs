use cmsis_interface::Q15;
use defmt::Format;
use fixed::types::I1F31;

use crate::adsr::{
    config_table::{
        ATTACK_BASE_COEFFICIENT_TABLE, DECAY_RELEASE_BASE_COEFFICIENT_TABLE,
        QUICK_RELEASE_BASE_COEFFICIENT,
    },
    sustain_amplitude_table::SUSTAIN_AMPLITUDE_TABLE,
};

pub mod config_table;
#[cfg(not(target_os = "none"))]
pub mod native_utils;
pub mod sustain_amplitude_table;

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

    pub(crate) fn decay(output: I1F31, base_and_coefficient: BaseAndCoefficient) -> I1F31 {
        output.saturating_mul_add(base_and_coefficient.coefficient, base_and_coefficient.base)
    }

    pub(crate) fn progress(&mut self, output: I1F31, adsr_config: &ADSRConfig) -> I1F31 {
        match *self {
            Self::Idle => I1F31::ZERO,
            Self::Attack => {
                let output = Self::decay(output, adsr_config.attack_base_and_coefficient);
                if output == I1F31::MAX {
                    *self = Self::Decay;
                }
                output
            }
            Self::Decay => {
                let output = Self::decay(output, adsr_config.decay_release_base_and_coefficient);
                if output <= adsr_config.sustain_level {
                    *self = Self::Sustain;
                    return adsr_config.sustain_level;
                }
                output
            }
            Self::Sustain => adsr_config.sustain_level,
            Self::Release => {
                let output = Self::decay(output, adsr_config.decay_release_base_and_coefficient);
                if output <= I1F31::ZERO {
                    *self = Self::Idle;
                    return I1F31::ZERO;
                }
                output
            }
            Self::QuickRelease => {
                let output = Self::decay(output, QUICK_RELEASE_BASE_COEFFICIENT);
                if output <= I1F31::ZERO {
                    *self = Self::Idle;
                    return I1F31::ZERO;
                }
                output
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ADSRConfig {
    sustain_level: I1F31,
    attack_base_and_coefficient: BaseAndCoefficient,
    decay_release_base_and_coefficient: BaseAndCoefficient,
}

impl ADSRConfig {
    pub(crate) fn new(sustain_config: u8, attack_config: u8, decay_release_config: u8) -> Self {
        Self {
            sustain_level: SUSTAIN_AMPLITUDE_TABLE[sustain_config as usize],
            attack_base_and_coefficient: ATTACK_BASE_COEFFICIENT_TABLE[attack_config as usize],
            decay_release_base_and_coefficient: DECAY_RELEASE_BASE_COEFFICIENT_TABLE
                [decay_release_config as usize],
        }
    }

    pub(crate) fn set_sustain(&mut self, sustain_config: u8) {
        self.sustain_level = SUSTAIN_AMPLITUDE_TABLE[sustain_config as usize];
    }

    pub(crate) fn set_attack(&mut self, attack_config: u8) {
        self.attack_base_and_coefficient = ATTACK_BASE_COEFFICIENT_TABLE[attack_config as usize];
    }

    pub(crate) fn set_decay_release(&mut self, decay_release_config: u8) {
        self.decay_release_base_and_coefficient =
            DECAY_RELEASE_BASE_COEFFICIENT_TABLE[decay_release_config as usize];
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ADSR {
    pub(crate) stage: ADSRStage,
    pub(crate) config: ADSRConfig,
    pub(crate) output: I1F31,
}

impl Format for ADSR {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ADSR {{ stage: {} }}", self.stage)
    }
}

impl ADSR {
    pub fn new(sustain_config: u8, attack_config: u8, decay_release_config: u8) -> ADSR {
        ADSR {
            stage: ADSRStage::Idle,
            config: ADSRConfig::new(sustain_config, attack_config, decay_release_config),
            output: I1F31::ZERO,
        }
    }

    pub fn play(&mut self) {
        self.output = I1F31::ZERO;
        self.stage.play()
    }

    pub fn retrigger(&mut self) {
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
            self.output = self.stage.progress(self.output, &self.config);
            *elem = fixed::traits::LossyInto::lossy_into(self.output);
        }
    }

    pub fn set_sustain(&mut self, sustain_config: u8) {
        self.config.set_sustain(sustain_config);
    }

    pub fn set_attack(&mut self, attack_config: u8) {
        self.config.set_attack(attack_config);
    }

    pub fn set_decay_release(&mut self, decay_release_config: u8) {
        self.config.set_decay_release(decay_release_config);
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
