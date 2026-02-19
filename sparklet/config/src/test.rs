use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};

use crate::{Config, ConfigEvent, ConfigManager};

#[test]
fn page_wrapping_forward_and_backward() {
    let signal = Signal::<NoopRawMutex, Config<8, 8>>::new();
    let mut manager = ConfigManager::new(&signal);

    manager.current_page = 0;
    manager.handle_event(ConfigEvent::PageChange { amount: -1 });
    assert_eq!(manager.current_page, 7);

    manager.current_page = 7;
    manager.handle_event(ConfigEvent::PageChange { amount: 1 });
    assert_eq!(manager.current_page, 0);
}

#[test]
fn encoder_saturation_at_boundaries() {
    let signal = Signal::<NoopRawMutex, Config<8, 8>>::new();
    let mut manager = ConfigManager::new(&signal);

    manager.config.pages[0].values[0] = 126;
    manager.handle_event(ConfigEvent::EncoderChange {
        encoder: 0,
        amount: -128,
    });
    assert_eq!(manager.config.pages[0].values[0], 0);

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

    signal.reset();

    manager.handle_event(ConfigEvent::PageChange { amount: 1 });
    assert!(!signal.signaled());

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

    manager.config.pages[0].values[2] = 100;

    manager.handle_event(ConfigEvent::EncoderChange {
        encoder: 10,
        amount: 10,
    });

    assert_eq!(manager.config.pages[0].values[2], 110);
}
