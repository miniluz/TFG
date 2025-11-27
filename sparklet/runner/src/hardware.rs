use defmt::info;

pub struct Hardware<'a> {
    #[cfg(feature = "midi-din")]
    pub midi_hardware: crate::midi_task::midi_din::hardware::MidiDinHardware<'a>,
}

impl<'a> Hardware<'a> {
    pub fn get() -> Hardware<'a> {
        info!("Initializing");
        let peripherals = embassy_stm32::init(Default::default());

        #[cfg(feature = "midi-din")]
        let midi_hardware = crate::get_midi_din_hardware!(peripherals);

        Hardware { midi_hardware }
    }
}
