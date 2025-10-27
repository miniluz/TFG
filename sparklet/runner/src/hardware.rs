use defmt::info;
use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals;
use embassy_stm32::usart::{self, RingBufferedUartRx, UartRx};
use static_cell::StaticCell;

pub struct Hardware<'a> {
    pub midi_uart_buffered: RingBufferedUartRx<'a>,
}

const MIDI_UART_BUFFER_SIZE: usize = 32;
static MIDI_UART_BUFFER: StaticCell<[u8; MIDI_UART_BUFFER_SIZE]> = StaticCell::new();

bind_interrupts!(struct Irqs {
    UART4 => usart::InterruptHandler<peripherals::UART4>;
});

impl<'a> Hardware<'a> {
    pub fn get() -> Hardware<'a> {
        info!("Initializing");
        let peripherals = embassy_stm32::init(Default::default());

        let mut config = usart::Config::default();
        config.baudrate = 31250;
        let midi_uart = UartRx::new(
            peripherals.UART4,
            Irqs,
            peripherals.PA1,
            peripherals.DMA1_CH0,
            config,
        );

        let midi_uart_buffer = MIDI_UART_BUFFER.init([0; MIDI_UART_BUFFER_SIZE]);

        let midi_uart_buffered = midi_uart.unwrap().into_ring_buffered(midi_uart_buffer);

        Hardware { midi_uart_buffered }
    }
}
