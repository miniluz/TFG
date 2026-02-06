#![cfg_attr(not(test), no_std)]

use defmt::Format;
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigEvent {
    PageChange { amount: i8 },
    EncoderChange { encoder: u8, amount: i8 },
}

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page<const ENCODER_AMOUNT: usize> {
    values: [u8; ENCODER_AMOUNT],
}

impl<const ENCODER_AMOUNT: usize> Page<ENCODER_AMOUNT> {
    fn new() -> Self {
        Self {
            values: [127; ENCODER_AMOUNT],
        }
    }
}

#[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config<const PAGE_AMOUNT: usize, const ENCODER_AMOUNT: usize> {
    pages: [Page<ENCODER_AMOUNT>; PAGE_AMOUNT],
}

impl<const PAGE_AMOUNT: usize, const ENCODER_AMOUNT: usize> Config<PAGE_AMOUNT, ENCODER_AMOUNT> {
    fn new() -> Self {
        Self {
            pages: [Page::<ENCODER_AMOUNT>::new(); PAGE_AMOUNT],
        }
    }
}

pub struct ConfigManager<'ch, M: RawMutex, const PAGE_AMOUNT: usize, const ENCODER_AMOUNT: usize> {
    signal: &'ch Signal<M, Config<PAGE_AMOUNT, ENCODER_AMOUNT>>,
    pub(crate) config: Config<PAGE_AMOUNT, ENCODER_AMOUNT>,
    pub(crate) current_page: usize,
}

impl<'ch, M: RawMutex, const PAGE_AMOUNT: usize, const ENCODER_AMOUNT: usize>
    ConfigManager<'ch, M, PAGE_AMOUNT, ENCODER_AMOUNT>
{
    pub fn new(signal: &'ch Signal<M, Config<PAGE_AMOUNT, ENCODER_AMOUNT>>) -> Self {
        assert!(PAGE_AMOUNT > 0);
        assert!(PAGE_AMOUNT <= 256);
        assert!(ENCODER_AMOUNT > 0);
        assert!(ENCODER_AMOUNT <= 256);

        let config: Config<_, _> = Config::new();
        if !signal.signaled() {
            signal.signal(config);
        }

        Self {
            signal,
            config,
            current_page: 0,
        }
    }

    fn signal_config(&self) {
        self.signal.signal(self.config)
    }

    pub fn handle_event(&mut self, event: ConfigEvent) {
        match event {
            ConfigEvent::PageChange { amount } => {
                self.current_page = (self.current_page as isize + amount as isize)
                    .rem_euclid(PAGE_AMOUNT as isize) as usize;
            }
            ConfigEvent::EncoderChange { encoder, amount } => {
                let encoder = &mut self.config.pages[self.current_page % PAGE_AMOUNT].values
                    [(encoder % ENCODER_AMOUNT as u8) as usize];
                *encoder = (*encoder).saturating_add_signed(amount);
                self.signal_config();
            }
        }
    }
}

#[cfg(test)]
mod test;
