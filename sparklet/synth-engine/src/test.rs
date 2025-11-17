use super::*;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use midi::u7;
use pretty_assertions::assert_eq;

// Constants for test voice bank and channel sizes
const TEST_VOICE_BANK_SIZE: usize = 4;
const TEST_CHANNEL_SIZE: usize = 4;

// Macro for easily setting up a SynthEngine instance with a Sender
macro_rules! setup_synth_engine {
    ($sender:ident, $se:ident) => {
        let channel = Channel::<NoopRawMutex, MidiEvent, TEST_CHANNEL_SIZE>::new();
        let $sender = channel.sender();
        let receiver = channel.receiver();
        let mut $se = SynthEngine::<'_, _, _, TEST_VOICE_BANK_SIZE>::new(receiver);
    };
}

// --- SynthEngine Tests ---

#[test]
fn synth_engine_render_samples_processes_note_on_events() {
    setup_synth_engine!(sender, se);

    let note = 60;
    let vel = 100;

    sender
        .try_send(MidiEvent::NoteOn { key: note, vel })
        .unwrap();

    let mut buffer = [Q15::ZERO; 10];
    se.render_samples(&mut buffer);

    // Verify MidiEvent was processed by VoiceBank
    assert_eq!(se.get_voice_bank_mut().count_active_voices(), 1);
    assert_eq!(se.get_voice_bank_mut().get_voice_note(0), note.into());
    assert_eq!(se.get_voice_bank_mut().get_voice_velocity(0), vel.into());
    assert_eq!(
        se.get_voice_bank_mut().get_voice_stage(0),
        VoiceStage::Held
    );
    assert_eq!(buffer, [Q15::ZERO; 10]); // Still zeroes buffer
}

#[test]
fn synth_engine_render_samples_processes_note_off_events() {
    setup_synth_engine!(sender, se);

    let note = 60;
    let vel = 100;

    // First NoteOn to get a voice active
    sender
        .try_send(MidiEvent::NoteOn { key: note, vel })
        .unwrap();
    let mut buffer = [Q15::ZERO; 10];
    se.render_samples(&mut buffer); // Process the NoteOn

    assert_eq!(se.get_voice_bank_mut().count_active_voices(), 1);

    // Now send NoteOff
    sender
        .try_send(MidiEvent::NoteOff { key: note, vel: 0 })
        .unwrap();
    se.render_samples(&mut buffer); // Process the NoteOff

    // Verify MidiEvent was processed by VoiceBank
    assert_eq!(se.get_voice_bank_mut().count_active_voices(), 0);
    assert_eq!(
        se.get_voice_bank_mut().get_voice_stage(0),
        VoiceStage::Free
    );
    assert_eq!(buffer, [Q15::ZERO; 10]);
}

#[test]
fn synth_engine_render_samples_processes_multiple_events() {
    setup_synth_engine!(sender, se);

    // NoteOn 60
    sender
        .try_send(MidiEvent::NoteOn { key: 60, vel: 100 })
        .unwrap();
    // NoteOn 62
    sender
        .try_send(MidiEvent::NoteOn { key: 62, vel: 100 })
        .unwrap();
    // NoteOff 60
    sender
        .try_send(MidiEvent::NoteOff { key: 60, vel: 0 })
        .unwrap();
    // NoteOn 64
    sender
        .try_send(MidiEvent::NoteOn { key: 64, vel: 100 })
        .unwrap();

    let mut buffer = [Q15::ZERO; 10];
    se.render_samples(&mut buffer);

    // Expected final state: 62 and 64 are held, 60 is free.
    assert_eq!(se.get_voice_bank_mut().count_active_voices(), 2);
    let vb = se.get_voice_bank_mut();
    let mut active_notes = Vec::new();
    for i in 0..TEST_VOICE_BANK_SIZE {
        if vb.get_voice_stage(i) == VoiceStage::Held {
            active_notes.push(vb.get_voice_note(i));
        }
    }
    active_notes.sort();
    assert_eq!(active_notes, vec![u7::from(62).into(), u7::from(64).into()]);
    assert_eq!(buffer, [Q15::ZERO; 10]);
}
