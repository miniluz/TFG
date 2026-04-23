use amity::triple::TripleBuffer;

use crate::{Config, ConfigEvent, ConfigManager, Page};

fn default_config() -> Config<8, 8> {
    Config {
        pages: [Page { values: [127; 8] }; 8],
    }
}

#[test]
fn page_wrapping_forward_and_backward() {
    let mut buffer = TripleBuffer::new(default_config(), default_config(), default_config());
    let (producer, _consumer) = buffer.split_mut();
    let mut manager = ConfigManager::new(producer);

    manager.current_page = 0;
    manager.handle_event(ConfigEvent::PageChange { amount: -1 });
    assert_eq!(manager.current_page, 7);

    manager.current_page = 7;
    manager.handle_event(ConfigEvent::PageChange { amount: 1 });
    assert_eq!(manager.current_page, 0);
}

#[test]
fn encoder_saturation_at_boundaries() {
    let mut buffer = TripleBuffer::new(default_config(), default_config(), default_config());
    let (producer, _consumer) = buffer.split_mut();
    let mut manager = ConfigManager::new(producer);

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
fn publishes_only_on_encoder_change() {
    let mut buffer = TripleBuffer::new(default_config(), default_config(), default_config());
    let (producer, _consumer) = buffer.split_mut();
    let mut manager = ConfigManager::new(producer);

    // No publish yet — PageChange does not publish
    let need_to_publish = manager.handle_event(ConfigEvent::PageChange { amount: 1 });
    assert!(!need_to_publish);

    // EncoderChange must publish
    let need_to_publish = manager.handle_event(ConfigEvent::EncoderChange {
        encoder: 0,
        amount: 1,
    });
    assert!(need_to_publish);
}

#[test]
fn encoder_index_modulo() {
    let mut buffer = TripleBuffer::new(default_config(), default_config(), default_config());
    let (producer, mut consumer) = buffer.split_mut();
    let mut manager = ConfigManager::new(producer);

    manager.config.pages[0].values[2] = 100;

    let need_to_publish = manager.handle_event(ConfigEvent::EncoderChange {
        encoder: 10,
        amount: 10,
    });

    assert_eq!(manager.config.pages[0].values[2], 110);

    // Consume and verify the published config reflects the change
    assert!(need_to_publish);
    manager.publish_config();
    consumer.consume();
    assert_eq!(consumer.get().pages[0].values[2], 110);
}
