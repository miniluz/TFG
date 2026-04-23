#![cfg_attr(not(test), no_std)]

use amity::triple::{TripleBuffer, TripleBufferProducer};
use defmt::Format;

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigEvent {
    PageChange { amount: i8 },
    EncoderChange { encoder: u8, amount: i8 },
}

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page<const ENCODER_AMOUNT: usize> {
    pub values: [u8; ENCODER_AMOUNT],
}

impl<const ENCODER_AMOUNT: usize> Page<ENCODER_AMOUNT> {
    pub fn new() -> Self {
        Self {
            values: [127; ENCODER_AMOUNT],
        }
    }

    pub fn from_config(config: [u8; ENCODER_AMOUNT]) -> Self {
        Self { values: config }
    }
}

impl<const ENCODER_AMOUNT: usize> Default for Page<ENCODER_AMOUNT> {
    fn default() -> Self {
        Self::new()
    }
}

/* Schema:
 *   First page: Attack, Sustain, Decay/Release
 *   Second page: Oscilator, unused, unused
 *   Third page and fourth page: Equalizer bank, from lowest to highest
 */
#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config<const PAGE_AMOUNT: usize, const ENCODER_AMOUNT: usize> {
    pub pages: [Page<ENCODER_AMOUNT>; PAGE_AMOUNT],
}

impl<const PAGE_AMOUNT: usize, const ENCODER_AMOUNT: usize> Config<PAGE_AMOUNT, ENCODER_AMOUNT> {
    pub fn new() -> Self {
        Self {
            pages: [Page::<ENCODER_AMOUNT>::new(); PAGE_AMOUNT],
        }
    }

    pub fn from_config(config: [[u8; ENCODER_AMOUNT]; PAGE_AMOUNT]) -> Self {
        Self {
            pages: config.map(Page::from_config),
        }
    }
}

impl<const PAGE_AMOUNT: usize, const ENCODER_AMOUNT: usize> Default
    for Config<PAGE_AMOUNT, ENCODER_AMOUNT>
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct ConfigManager<'buf, const PAGE_AMOUNT: usize, const ENCODER_AMOUNT: usize> {
    producer: TripleBufferProducer<
        Config<PAGE_AMOUNT, ENCODER_AMOUNT>,
        &'buf TripleBuffer<Config<PAGE_AMOUNT, ENCODER_AMOUNT>>,
    >,
    pub(crate) config: Config<PAGE_AMOUNT, ENCODER_AMOUNT>,
    pub(crate) current_page: usize,
}

impl<'buf, const PAGE_AMOUNT: usize, const ENCODER_AMOUNT: usize>
    ConfigManager<'buf, PAGE_AMOUNT, ENCODER_AMOUNT>
{
    pub fn new(
        producer: TripleBufferProducer<
            Config<PAGE_AMOUNT, ENCODER_AMOUNT>,
            &'buf TripleBuffer<Config<PAGE_AMOUNT, ENCODER_AMOUNT>>,
        >,
    ) -> Self {
        assert!(PAGE_AMOUNT > 0);
        assert!(PAGE_AMOUNT <= 256);
        assert!(ENCODER_AMOUNT > 0);
        assert!(ENCODER_AMOUNT <= 256);

        Self {
            producer,
            config: Config::new(),
            current_page: 0,
        }
    }

    pub fn from_config(
        producer: TripleBufferProducer<
            Config<PAGE_AMOUNT, ENCODER_AMOUNT>,
            &'buf TripleBuffer<Config<PAGE_AMOUNT, ENCODER_AMOUNT>>,
        >,
        config: [[u8; ENCODER_AMOUNT]; PAGE_AMOUNT],
    ) -> Self {
        assert!(PAGE_AMOUNT > 0);
        assert!(PAGE_AMOUNT <= 256);
        assert!(ENCODER_AMOUNT > 0);
        assert!(ENCODER_AMOUNT <= 256);

        Self {
            producer,
            config: Config::from_config(config),
            current_page: 0,
        }
    }

    pub fn publish_config(&mut self) {
        *self.producer.get_mut() = self.config;
        self.producer.publish();
    }

    pub fn handle_event(&mut self, event: ConfigEvent) -> bool {
        let mut need_to_publish = false;
        match event {
            ConfigEvent::PageChange { amount } => {
                self.current_page = (self.current_page as isize + amount as isize)
                    .rem_euclid(PAGE_AMOUNT as isize) as usize;
            }
            ConfigEvent::EncoderChange { encoder, amount } => {
                let encoder = &mut self.config.pages[self.current_page % PAGE_AMOUNT].values
                    [(encoder % ENCODER_AMOUNT as u8) as usize];
                *encoder = (*encoder).saturating_add_signed(amount);

                need_to_publish = true;
            }
        }
        defmt::info!("Event triggered: {}", event);
        defmt::info!(
            "Current page: {}\nConfig: {}",
            self.current_page,
            self.config
        );

        need_to_publish
    }
}

#[cfg(test)]
mod test;
