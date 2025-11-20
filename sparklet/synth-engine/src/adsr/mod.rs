use cmsis_interface::Q15;
use fixed::types::I1F31;

enum ADSRStage {
    IDLE,
    ATTACK,
    DECAY,
    SUSTAIN,
    RELEASE,
}

impl ADSRStage {
    fn play(&mut self) {
        *self = Self::ATTACK;
    }

    fn stop_playing(&mut self) {
        match *self {
            Self::IDLE => (),
            _ => {
                *self = ADSRStage::RELEASE;
            }
        }
    }

    fn progress(&mut self, output: I1F31, adsr_config: &ADSRConfig) -> I1F31 {
        match *self {
            Self::IDLE => I1F31::ZERO,
            Self::ATTACK => {
                let output = output
                    .saturating_mul_add(adsr_config.attack_coefficient, adsr_config.attack_base);
                if output == I1F31::MAX {
                    *self = Self::DECAY;
                }
                output
            }
            Self::DECAY => {
                let output = output.saturating_mul_add(
                    adsr_config.decay_release_coefficient,
                    adsr_config.decay_release_base,
                );
                if output <= adsr_config.sustain_level {
                    *self = Self::SUSTAIN;
                    return adsr_config.sustain_level;
                }
                output
            }
            Self::SUSTAIN => adsr_config.sustain_level,
            Self::RELEASE => {
                let output = output.saturating_mul_add(
                    adsr_config.decay_release_coefficient,
                    adsr_config.decay_release_base,
                );
                if output <= I1F31::ZERO {
                    *self = Self::SUSTAIN;
                    return I1F31::ZERO;
                }
                output
            }
        }
    }
}

struct ADSRConfig {
    sustain_level: I1F31,
    attack_base: I1F31,
    attack_coefficient: I1F31,
    decay_release_base: I1F31,
    decay_release_coefficient: I1F31,
}

struct ADSR {
    stage: ADSRStage,
    config: ADSRConfig,
    output: I1F31,
}

impl ADSR {
    pub fn play(&mut self) {
        self.stage.play()
    }

    pub fn stop_playing(&mut self) {
        self.stage.stop_playing()
    }

    pub fn get_samples<const LEN: usize>(&mut self, buffer: &mut [Q15; LEN]) {
        for i in 0..LEN {
            buffer[i] = fixed::traits::LossyInto::lossy_into(
                self.stage.progress(self.output, &self.config),
            );
        }
    }
}
