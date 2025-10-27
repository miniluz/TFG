use super::*;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use midi::u7;
use pretty_assertions::assert_eq;

// Constants for test voice bank and channel sizes
const TEST_VOICE_BANK_SIZE: usize = 4;
const TEST_CHANNEL_SIZE: usize = 4;

// Macro for easily setting up a VoiceBank instance
macro_rules! setup_voice_bank {
    ($vb:ident) => {
        #[allow(unused_mut)]
        let mut $vb = VoiceBank::<TEST_VOICE_BANK_SIZE>::new();
    };
}

// Macro for easily setting up a SynthEngine instance with a Sender
macro_rules! setup_synth_engine {
    ($sender:ident, $se:ident) => {
        let channel = Channel::<NoopRawMutex, MidiEvent, TEST_CHANNEL_SIZE>::new();
        let $sender = channel.sender();
        let receiver = channel.receiver();
        let mut $se = SynthEngine::<'_, _, _, TEST_VOICE_BANK_SIZE>::new(receiver);
    };
}

// --- VoiceBank Tests ---

#[test]
fn voice_bank_new_initializes_all_voices_as_free() {
    setup_voice_bank!(vb);

    assert_eq!(vb.count_active_voices(), 0);
    for i in 0..TEST_VOICE_BANK_SIZE {
        assert_eq!(vb.get_voice_state(i).stage, VoiceStage::Free);
    }
}

#[test]
fn voice_bank_play_note_fills_free_voices() {
    setup_voice_bank!(vb);

    for i in 0..TEST_VOICE_BANK_SIZE {
        vb.play_note(i as u8, 100);
        assert_eq!(vb.count_active_voices(), i + 1);
        assert_eq!(vb.get_voice_state(i).note, u7::from(i as u8));
        assert_eq!(vb.get_voice_state(i).velocity, u7::from(100));
        assert_eq!(vb.get_voice_state(i).stage, VoiceStage::Held);
    }
    assert_eq!(vb.count_active_voices(), TEST_VOICE_BANK_SIZE);
}

#[test]
fn voice_bank_play_note_steals_earliest_voice_when_full() {
    setup_voice_bank!(vb);

    // Fill all voices
    for i in 0..TEST_VOICE_BANK_SIZE {
        vb.play_note(i as u8, 100);
    }
    assert_eq!(vb.count_active_voices(), TEST_VOICE_BANK_SIZE);

    // Play one more note, it should steal the voice at index 0 (note 0)
    // because all start times are 0.
    let stolen_note = TEST_VOICE_BANK_SIZE as u8;
    vb.play_note(stolen_note, 120);

    assert_eq!(vb.count_active_voices(), TEST_VOICE_BANK_SIZE); // Still full
    assert_eq!(vb.get_voice_state(0).note, stolen_note);
    assert_eq!(vb.get_voice_state(0).velocity, u7::from(120));
    assert_eq!(vb.get_voice_state(0).stage, VoiceStage::Held);

    // Other voices should be untouched
    for i in 1..TEST_VOICE_BANK_SIZE {
        assert_eq!(vb.get_voice_state(i).note, u7::from(i as u8));
        assert_eq!(vb.get_voice_state(i).velocity, u7::from(100));
        assert_eq!(vb.get_voice_state(i).stage, VoiceStage::Held);
    }
}

#[test]
fn voice_bank_release_note_frees_active_note() {
    setup_voice_bank!(vb);

    let note_to_play = 60;
    vb.play_note(note_to_play, 100);
    assert_eq!(vb.count_active_voices(), 1);

    vb.release_note(note_to_play);
    assert_eq!(vb.count_active_voices(), 0);
    assert_eq!(vb.get_voice_state(0).stage, VoiceStage::Free);
}

#[test]
fn voice_bank_release_note_frees_all_instances_of_a_note() {
    setup_voice_bank!(vb);

    let note_to_play = 60;
    vb.play_note(note_to_play, 100); // Voice 0
    vb.play_note(note_to_play, 90); // Voice 1 (if TEST_VOICE_BANK_SIZE >= 2)
    assert_eq!(vb.count_active_voices(), 2);

    vb.release_note(note_to_play);
    assert_eq!(vb.count_active_voices(), 0);
    assert_eq!(vb.get_voice_state(0).stage, VoiceStage::Free);
    assert_eq!(vb.get_voice_state(1).stage, VoiceStage::Free);
}

#[test]
fn voice_bank_release_note_non_existent_does_nothing() {
    setup_voice_bank!(vb);

    vb.play_note(60, 100);
    vb.play_note(62, 100);
    assert_eq!(vb.count_active_voices(), 2);

    // Try to release a note that was not played
    vb.release_note(64);
    assert_eq!(vb.count_active_voices(), 2);
    assert_eq!(vb.get_voice_state(0).note, u7::from(60));
    assert_eq!(vb.get_voice_state(1).note, u7::from(62));
    assert_eq!(vb.get_voice_state(0).stage, VoiceStage::Held);
    assert_eq!(vb.get_voice_state(1).stage, VoiceStage::Held);
}

#[test]
fn voice_bank_process_midi_event_handles_note_on_and_off() {
    setup_voice_bank!(vb);

    let note = 60;
    let vel = 100;

    // NoteOn
    vb.process_midi_event(MidiEvent::NoteOn { key: note, vel });
    assert_eq!(vb.count_active_voices(), 1);
    assert_eq!(vb.get_voice_state(0).note, note);
    assert_eq!(vb.get_voice_state(0).velocity, vel);
    assert_eq!(vb.get_voice_state(0).stage, VoiceStage::Held);

    // NoteOff
    vb.process_midi_event(MidiEvent::NoteOff { key: note, vel: 0 });
    assert_eq!(vb.count_active_voices(), 0);
    assert_eq!(vb.get_voice_state(0).stage, VoiceStage::Free);
}

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
    assert_eq!(se.get_voice_bank_mut().get_voice_state(0).note, note);
    assert_eq!(se.get_voice_bank_mut().get_voice_state(0).velocity, vel);
    assert_eq!(
        se.get_voice_bank_mut().get_voice_state(0).stage,
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
        se.get_voice_bank_mut().get_voice_state(0).stage,
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
        if vb.get_voice_state(i).stage == VoiceStage::Held {
            active_notes.push(vb.get_voice_state(i).note);
        }
    }
    active_notes.sort();
    assert_eq!(active_notes, vec![u7::from(62), u7::from(64)]);
    assert_eq!(buffer, [Q15::ZERO; 10]);
}
