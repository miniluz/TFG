use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};

use crate::{Config, ConfigEvent, ConfigManager};

#[test]
fn page_wrapping_forward_and_backward() {
    let signal = Signal::<NoopRawMutex, Config<8, 8>>::new();
    let mut manager = ConfigManager::new(&signal);

    // Start at page 0, change by -1 → should wrap to page 7
    manager.current_page = 0;
    manager.handle_event(ConfigEvent::PageChange { amount: -1 });
    assert_eq!(manager.current_page, 7);

    // Start at page 7, change by +1 → should wrap to page 0
    manager.current_page = 7;
    manager.handle_event(ConfigEvent::PageChange { amount: 1 });
    assert_eq!(manager.current_page, 0);
}

#[test]
fn encoder_saturation_at_boundaries() {
    let signal = Signal::<NoopRawMutex, Config<8, 8>>::new();
    let mut manager = ConfigManager::new(&signal);

    // Set encoder to 126, add -128 → should saturate to 0
    manager.config.pages[0].values[0] = 126;
    manager.handle_event(ConfigEvent::EncoderChange {
        encoder: 0,
        amount: -128,
    });
    assert_eq!(manager.config.pages[0].values[0], 0);

    // Set encoder to 129, add 127 → should saturate to 255
    manager.config.pages[0].values[1] = 129;
    manager.handle_event(ConfigEvent::EncoderChange {
        encoder: 1,
        amount: 127,
    });
    assert_eq!(manager.config.pages[0].values[1], 255);
}

#[test]
fn signal_only_on_encoder_change() {
    let signal = Signal::<NoopRawMutex, Config<8, 8>>::new();
    let mut manager = ConfigManager::new(&signal);

    // Clear any initial signal
    signal.reset();

    // PageChange event → verify no signal
    manager.handle_event(ConfigEvent::PageChange { amount: 1 });
    assert!(!signal.signaled());

    // EncoderChange event → verify signal sent
    manager.handle_event(ConfigEvent::EncoderChange {
        encoder: 0,
        amount: 1,
    });
    assert!(signal.signaled());
}

#[test]
fn encoder_index_modulo() {
    let signal = Signal::<NoopRawMutex, Config<8, 8>>::new();
    let mut manager = ConfigManager::new(&signal);

    // Set encoder 2 to a known value
    manager.config.pages[0].values[2] = 100;

    // Trigger EncoderChange with encoder=10 (> 8) → should use encoder % 8 = 2
    manager.handle_event(ConfigEvent::EncoderChange {
        encoder: 10,
        amount: 10,
    });

    // Verify encoder 2 was modified (100 + 10 = 110)
    assert_eq!(manager.config.pages[0].values[2], 110);
}
