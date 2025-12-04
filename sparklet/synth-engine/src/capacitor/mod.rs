use fixed::types::I1F31;

use crate::adsr::{BaseAndCoefficient, config_table::QUICK_FALL_BASE_COEFFICIENT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacitorStatus {
    ReachedTarget,
    Charging,
    Discharging,
    QuickDischarging,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capacitor {
    current: I1F31,
    target: I1F31,
    rise_coeff: BaseAndCoefficient,
    fall_coeff: BaseAndCoefficient,
    status: CapacitorStatus,
}

impl Capacitor {
    pub fn new(rise_coeff: BaseAndCoefficient, fall_coeff: BaseAndCoefficient) -> Self {
        Self {
            current: I1F31::ZERO,
            target: I1F31::ZERO,
            rise_coeff,
            fall_coeff,
            status: CapacitorStatus::ReachedTarget,
        }
    }

    pub fn set_target(&mut self, target: I1F31) {
        self.target = target;

        if self.current < target {
            self.status = CapacitorStatus::Charging;
        } else if self.current > target {
            self.status = CapacitorStatus::Discharging;
        } else {
            self.status = CapacitorStatus::ReachedTarget;
        }
    }

    pub fn set_level(&mut self, level: I1F31) {
        self.current = level;
    }

    pub fn get_level(&self) -> I1F31 {
        self.current
    }

    pub fn set_rise_coeff(&mut self, rise_coeff: BaseAndCoefficient) {
        self.rise_coeff = rise_coeff;
    }

    pub fn set_fall_coeff(&mut self, fall_coeff: BaseAndCoefficient) {
        self.fall_coeff = fall_coeff;
    }

    pub fn quick_discharge(&mut self) {
        self.target = I1F31::ZERO;
        self.status = CapacitorStatus::QuickDischarging;
    }

    pub(crate) fn iterate(value: I1F31, coeff: BaseAndCoefficient) -> I1F31 {
        value.saturating_mul_add(coeff.coefficient, coeff.base)
    }

    pub fn step(&mut self) -> CapacitorStatus {
        match self.status {
            CapacitorStatus::ReachedTarget => self.status,
            CapacitorStatus::Charging => {
                let next = Self::iterate(self.current, self.rise_coeff);

                if next >= self.target {
                    self.current = self.target;
                    self.status = CapacitorStatus::ReachedTarget;
                } else {
                    self.current = next;
                }
                self.status
            }
            CapacitorStatus::Discharging => {
                let next = Self::iterate(self.current, self.fall_coeff);

                if next <= self.target {
                    self.current = self.target;
                    self.status = CapacitorStatus::ReachedTarget;
                } else {
                    self.current = next;
                }
                self.status
            }
            CapacitorStatus::QuickDischarging => {
                let next = Self::iterate(self.current, QUICK_FALL_BASE_COEFFICIENT);

                if next <= self.target {
                    self.current = self.target;
                    self.status = CapacitorStatus::ReachedTarget;
                } else {
                    self.current = next;
                }
                self.status
            }
        }
    }
}

impl Default for Capacitor {
    fn default() -> Self {
        Self::new(
            BaseAndCoefficient {
                base: I1F31::ZERO,
                coefficient: I1F31::ZERO,
            },
            BaseAndCoefficient {
                base: I1F31::ZERO,
                coefficient: I1F31::ZERO,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adsr::config_table::FALL_BASE_COEFFICIENT_TABLE;

    #[test]
    fn test_capacitor_rises_to_target() {
        // Use coefficients that work with I1F31 (coefficient < 1.0)
        let rise_coeff = BaseAndCoefficient {
            base: I1F31::from_num(0.1),
            coefficient: I1F31::from_bits(0x7f000000), // ~0.996
        };
        let fall_coeff = BaseAndCoefficient {
            base: I1F31::ZERO,
            coefficient: I1F31::from_bits(0x7f000000),
        };

        let mut cap = Capacitor::new(rise_coeff, fall_coeff);
        cap.set_target(I1F31::from_num(0.5));

        let mut status = cap.step();
        assert_eq!(status, CapacitorStatus::Charging);
        assert!(cap.get_level() > I1F31::ZERO);
        assert!(cap.get_level() < I1F31::from_num(0.5));

        // Step until we reach the target
        let mut iterations = 0;
        while status != CapacitorStatus::ReachedTarget && iterations < 100 {
            status = cap.step();
            iterations += 1;
        }

        assert_eq!(status, CapacitorStatus::ReachedTarget);
        assert_eq!(cap.get_level(), I1F31::from_num(0.5));
    }

    #[test]
    fn test_capacitor_falls_to_target() {
        let rise_coeff = BaseAndCoefficient {
            base: I1F31::ZERO,
            coefficient: I1F31::from_bits(0x7f000000),
        };
        // Subtract 0.1 each step
        let fall_coeff = BaseAndCoefficient {
            base: I1F31::from_num(-0.1),
            coefficient: I1F31::from_bits(0x7f000000),
        };

        let mut cap = Capacitor::new(rise_coeff, fall_coeff);
        cap.set_level(I1F31::MAX);
        cap.set_target(I1F31::from_num(0.5));

        let mut status = cap.step();
        assert_eq!(status, CapacitorStatus::Discharging);
        assert!(cap.get_level() < I1F31::MAX);
        assert!(cap.get_level() > I1F31::from_num(0.5));

        // Step until we reach the target
        let mut iterations = 0;
        while status != CapacitorStatus::ReachedTarget && iterations < 100 {
            status = cap.step();
            iterations += 1;
        }

        assert_eq!(status, CapacitorStatus::ReachedTarget);
        assert_eq!(cap.get_level(), I1F31::from_num(0.5));
    }

    #[test]
    fn test_capacitor_does_not_overshoot_rising() {
        // Large step that would overshoot
        let rise_coeff = BaseAndCoefficient {
            base: I1F31::from_num(0.1),
            coefficient: I1F31::from_bits(0x7f000000),
        };
        let fall_coeff = BaseAndCoefficient {
            base: I1F31::ZERO,
            coefficient: I1F31::from_bits(0x7f000000),
        };

        let mut cap = Capacitor::new(rise_coeff, fall_coeff);
        cap.set_level(I1F31::from_num(0.45));
        cap.set_target(I1F31::from_num(0.5));

        let status = cap.step();
        assert_eq!(status, CapacitorStatus::ReachedTarget);
        assert_eq!(cap.get_level(), I1F31::from_num(0.5));
    }

    #[test]
    fn test_capacitor_does_not_overshoot_falling() {
        let rise_coeff = BaseAndCoefficient {
            base: I1F31::ZERO,
            coefficient: I1F31::from_bits(0x7f000000),
        };
        // Large step that would overshoot
        let fall_coeff = BaseAndCoefficient {
            base: I1F31::from_num(-0.1),
            coefficient: I1F31::from_bits(0x7f000000),
        };

        let mut cap = Capacitor::new(rise_coeff, fall_coeff);
        cap.set_level(I1F31::from_num(0.55));
        cap.set_target(I1F31::from_num(0.5));

        let status = cap.step();
        assert_eq!(status, CapacitorStatus::ReachedTarget);
        assert_eq!(cap.get_level(), I1F31::from_num(0.5));
    }

    #[test]
    fn test_capacitor_already_at_target() {
        let rise_coeff = BaseAndCoefficient {
            base: I1F31::from_num(0.1),
            coefficient: I1F31::from_bits(0x7f000000),
        };
        let fall_coeff = BaseAndCoefficient {
            base: I1F31::from_num(-0.1),
            coefficient: I1F31::from_bits(0x7f000000),
        };

        let mut cap = Capacitor::new(rise_coeff, fall_coeff);
        cap.set_level(I1F31::from_num(0.5));
        cap.set_target(I1F31::from_num(0.5));

        let status = cap.step();
        assert_eq!(status, CapacitorStatus::ReachedTarget);
        assert_eq!(cap.get_level(), I1F31::from_num(0.5));
    }

    #[test]
    fn test_capacitor_target_change_mid_flight() {
        let rise_coeff = BaseAndCoefficient {
            base: I1F31::from_num(0.1),
            coefficient: I1F31::from_bits(0x7f000000),
        };
        let fall_coeff = BaseAndCoefficient {
            base: I1F31::from_num(-0.1),
            coefficient: I1F31::from_bits(0x7f000000),
        };

        let mut cap = Capacitor::new(rise_coeff, fall_coeff);
        cap.set_target(I1F31::MAX);

        // Rise for a few steps
        cap.step();
        cap.step();
        cap.step();
        cap.step();
        let mid_level = cap.get_level();
        assert!(mid_level > I1F31::ZERO);
        assert!(mid_level < I1F31::MAX);

        // Change target to something significantly lower
        cap.set_target(I1F31::from_num(0.05));

        // Should now start falling
        let status = cap.step();
        assert_eq!(status, CapacitorStatus::Discharging);
        assert!(cap.get_level() < mid_level);
    }

    #[test]
    fn test_capacitor_quick_discharge() {
        let rise_coeff = BaseAndCoefficient {
            base: I1F31::from_num(0.1),
            coefficient: I1F31::from_bits(0x7f000000),
        };
        // Use fall coefficient at index 1 (slower than index 0 which is the quick fall)
        let slow_fall_coeff = FALL_BASE_COEFFICIENT_TABLE[1];

        // Test 1: Quick discharge from MAX to zero
        let mut cap_quick = Capacitor::new(rise_coeff, slow_fall_coeff);
        cap_quick.set_level(I1F31::MAX);
        cap_quick.quick_discharge();

        // Should be in QuickDischarging state
        let status = cap_quick.step();
        assert_eq!(status, CapacitorStatus::QuickDischarging);

        // Should be discharging towards zero
        assert!(cap_quick.get_level() < I1F31::MAX);

        // Count iterations for quick discharge
        let mut quick_iterations = 1; // Already did one step
        let mut final_status = status;
        while final_status != CapacitorStatus::ReachedTarget && quick_iterations < 1000 {
            final_status = cap_quick.step();
            quick_iterations += 1;
        }
        assert_eq!(final_status, CapacitorStatus::ReachedTarget);
        assert_eq!(cap_quick.get_level(), I1F31::ZERO);

        // Test 2: Normal slow discharge from MAX to zero
        let mut cap_slow = Capacitor::new(rise_coeff, slow_fall_coeff);
        cap_slow.set_level(I1F31::MAX);
        cap_slow.set_target(I1F31::ZERO); // Normal discharge

        // Count iterations for slow discharge
        let mut slow_iterations = 0;
        let mut slow_status = cap_slow.step();
        while slow_status != CapacitorStatus::ReachedTarget && slow_iterations < 10000 {
            slow_status = cap_slow.step();
            slow_iterations += 1;
        }
        assert_eq!(slow_status, CapacitorStatus::ReachedTarget);
        assert_eq!(cap_slow.get_level(), I1F31::ZERO);

        // Quick discharge should be significantly faster than slow discharge
        assert!(
            quick_iterations < slow_iterations,
            "Quick discharge took {} iterations, but slow discharge took {} iterations",
            quick_iterations,
            slow_iterations
        );
    }
}
