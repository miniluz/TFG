use defmt::trace;
use embassy_executor::SpawnToken;
use embassy_stm32::usart::RingBufferedUartRx;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use midi::{MidiEvent, MidiListener};
use static_cell::StaticCell;

pub const MIDI_CHANNEL_SIZE: usize = 16;

pub struct MidiTaskState<'a> {
    midi_listener: MidiListener<'a, CriticalSectionRawMutex, MIDI_CHANNEL_SIZE>,
    midi_uart_buffered: RingBufferedUartRx<'a>,
}

impl<'a> MidiTaskState<'a> {
    pub fn new(
        midi_listener: MidiListener<'a, CriticalSectionRawMutex, MIDI_CHANNEL_SIZE>,
        midi_uart_buffered: RingBufferedUartRx<'a>,
    ) -> MidiTaskState<'a> {
        MidiTaskState {
            midi_listener,
            midi_uart_buffered,
        }
    }
}

pub static MIDI_TASK_STATE: StaticCell<MidiTaskState> = StaticCell::new();

pub static MIDI_TASK_CHANNEL: Channel<CriticalSectionRawMutex, MidiEvent, MIDI_CHANNEL_SIZE> =
    Channel::new();

pub fn create_task(midi_uart_buffered: RingBufferedUartRx<'static>) -> SpawnToken<impl Sized> {
    let midi_sender = MIDI_TASK_CHANNEL.sender();

    let midi_listener = MidiListener::new(midi_sender);

    midi_task(MIDI_TASK_STATE.init(MidiTaskState::new(midi_listener, midi_uart_buffered)))
}

#[embassy_executor::task]
pub async fn midi_task(state: &'static mut MidiTaskState<'static>) {
    let mut buffer = [0; 1];
    loop {
        match state.midi_uart_buffered.read(&mut buffer).await {
            Ok(1) => state.midi_listener.process_bytes(&buffer),
            Ok(other_size) => {
                trace!(
                    "Unexpected number of bytes read on MIDI UART: {}",
                    other_size
                );
            }
            Err(err) => {
                trace!("Error reading from MIDI UART: {}", err);
            }
        };
    }
}
