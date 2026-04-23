use crate::build_config::BUILD_CONFIG;
use amity::triple::{TripleBuffer, TripleBufferConsumer, TripleBufferProducer};
use config::Config;
use static_cell::StaticCell;

#[cfg(feature = "configurable")]
pub mod task;

const BASE_PAGE_COUNT: usize = 2;

#[cfg(feature = "octave-filter")]
const OCTAVE_FILTER_PAGE_COUNT: usize = 2;
#[cfg(not(feature = "octave-filter"))]
const OCTAVE_FILTER_PAGE_COUNT: usize = 0;

#[cfg(not(feature = "octave-filter"))]
pub const INITIAL_CONFIG: [[u8; CONFIG_ENCODER_COUNT]; CONFIG_PAGE_COUNT] = [
    [
        BUILD_CONFIG.initial_config.attack,
        BUILD_CONFIG.initial_config.sustain,
        BUILD_CONFIG.initial_config.decay_release,
    ],
    [BUILD_CONFIG.initial_config.oscilator_type, 127, 127],
];

#[cfg(feature = "octave-filter")]
pub const INITIAL_CONFIG: [[u8; CONFIG_ENCODER_COUNT]; CONFIG_PAGE_COUNT] = [
    [
        BUILD_CONFIG.initial_config.attack,
        BUILD_CONFIG.initial_config.sustain,
        BUILD_CONFIG.initial_config.decay_release,
    ],
    [BUILD_CONFIG.initial_config.oscilator_type, 127, 127],
    [
        BUILD_CONFIG.initial_config.f250hz,
        BUILD_CONFIG.initial_config.f500hz,
        BUILD_CONFIG.initial_config.f1000hz,
    ],
    [
        BUILD_CONFIG.initial_config.f2000hz,
        BUILD_CONFIG.initial_config.f4000hz,
        BUILD_CONFIG.initial_config.f8000hz,
    ],
];

pub const CONFIG_PAGE_COUNT: usize = BASE_PAGE_COUNT + OCTAVE_FILTER_PAGE_COUNT;
pub const CONFIG_ENCODER_COUNT: usize = 3;

// Type aliases for the triple buffer halves
type ConfigBuffer = TripleBuffer<Config<CONFIG_PAGE_COUNT, CONFIG_ENCODER_COUNT>>;
pub type ConfigProducer =
    TripleBufferProducer<Config<CONFIG_PAGE_COUNT, CONFIG_ENCODER_COUNT>, &'static ConfigBuffer>;
pub type ConfigConsumer =
    TripleBufferConsumer<Config<CONFIG_PAGE_COUNT, CONFIG_ENCODER_COUNT>, &'static ConfigBuffer>;

// Static triple buffer for config transport
static CONFIG_TRIPLE_BUFFER: StaticCell<ConfigBuffer> = StaticCell::new();

/// Initialise the config triple buffer and return the producer/consumer halves.
/// Must be called exactly once before creating the config manager or synth engine tasks.
pub fn init_config_transport() -> (ConfigProducer, ConfigConsumer) {
    let initial_config = Config::from_config(INITIAL_CONFIG);

    let buf = CONFIG_TRIPLE_BUFFER.init(TripleBuffer::new(
        initial_config,
        initial_config,
        initial_config,
    ));
    buf.split_mut()
}

#[cfg(not(feature = "configurable"))]
pub fn send_initial_config(mut producer: ConfigProducer) {
    let config = Config::from_config(INITIAL_CONFIG);

    *producer.get_mut() = config;
    producer.publish();
}
