use embassy_stm32::peripherals;
use embassy_stm32::usb;
use embassy_usb::class::midi::MidiClass;

pub struct MidiUsbHardware<'d> {
    pub midi_class: MidiClass<'d, usb::Driver<'d, peripherals::USB_OTG_HS>>,
}

#[macro_export]
macro_rules! get_midi_usb_hardware {
    ($builder:expr) => {{
        // Create MIDI class with 1 input jack, 1 output jack, 64 byte packets
        let midi_class = embassy_usb::class::midi::MidiClass::new(
            $builder, 1,  // num input jacks
            1,  // num output jacks
            64, // max packet size
        );

        $crate::midi_task::midi_usb::hardware::MidiUsbHardware { midi_class }
    }};
}
