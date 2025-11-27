use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals;
use embassy_stm32::usart::{self, RingBufferedUartRx};
use static_cell::StaticCell;

pub struct MidiDinHardware<'a> {
    pub midi_uart_buffered: RingBufferedUartRx<'a>,
}

pub const MIDI_UART_BUFFER_SIZE: usize = 32;
pub static MIDI_UART_BUFFER: StaticCell<[u8; MIDI_UART_BUFFER_SIZE]> = StaticCell::new();

bind_interrupts!(pub struct Irqs {
    UART4 => usart::InterruptHandler<peripherals::UART4>;
});

#[macro_export]
macro_rules! get_midi_din_hardware {
    ($peripherals:ident) => {{
        let mut config = embassy_stm32::usart::Config::default();
        config.baudrate = 31250;
        let midi_uart = embassy_stm32::usart::UartRx::new(
            $peripherals.UART4,
            $crate::midi_task::midi_din::hardware::Irqs,
            $peripherals.PA1,
            $peripherals.DMA1_CH0,
            config,
        );

        let midi_uart_buffer = $crate::midi_task::midi_din::hardware::MIDI_UART_BUFFER
            .init([0; $crate::midi_task::midi_din::hardware::MIDI_UART_BUFFER_SIZE]);

        let midi_uart_buffered = midi_uart.unwrap().into_ring_buffered(midi_uart_buffer);

        $crate::midi_task::midi_din::hardware::MidiDinHardware { midi_uart_buffered }
    }};
}
