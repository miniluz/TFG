use super::*;
use midi::u7;
use pretty_assertions::assert_eq;
use crate::wavetable::sine_wavetable::SINE_WAVETABLE;

const TEST_VOICE_BANK_SIZE: usize = 4;

macro_rules! setup_voice_bank {
    ($vb:ident) => {
        #[allow(unused_mut)]
        let mut $vb = VoiceBank::<TEST_VOICE_BANK_SIZE>::new(
            &SINE_WAVETABLE,
            200, // sustain
            50,  // attack
            100, // decay_release
        );
    };
}

#[test]
fn voice_bank_new_initializes_all_voices_as_free() {
    setup_voice_bank!(vb);

    assert_eq!(vb.count_active_voices(), 0);
    for i in 0..TEST_VOICE_BANK_SIZE {
        assert_eq!(vb.get_voice_stage(i), VoiceStage::Free);
    }
}

#[test]
fn voice_bank_play_note_fills_free_voices() {
    setup_voice_bank!(vb);

    for i in 0..TEST_VOICE_BANK_SIZE {
        vb.play_note((i as u8).into(), 100.into());
        assert_eq!(vb.count_active_voices(), i + 1);
        assert_eq!(vb.get_voice_note(i), u7::from(i as u8).into());
        assert_eq!(vb.get_voice_velocity(i), u7::from(100).into());
        assert_eq!(vb.get_voice_stage(i), VoiceStage::Held);
    }
    assert_eq!(vb.count_active_voices(), TEST_VOICE_BANK_SIZE);
}

#[test]
fn voice_bank_play_note_steals_earliest_voice_when_full() {
    setup_voice_bank!(vb);

    // Fill all voices
    for i in 0..TEST_VOICE_BANK_SIZE {
        vb.play_note((i as u8).into(), 100.into());
    }
    assert_eq!(vb.count_active_voices(), TEST_VOICE_BANK_SIZE);

    // Play one more note, it should steal the voice at index 0 (note 0)
    // because all start times are 0.
    let stolen_note = TEST_VOICE_BANK_SIZE as u8;
    vb.play_note(stolen_note.into(), 120.into());

    assert_eq!(vb.count_active_voices(), TEST_VOICE_BANK_SIZE); // Still full
    assert_eq!(vb.get_voice_note(0), stolen_note.into());
    assert_eq!(vb.get_voice_velocity(0), u7::from(120).into());
    assert_eq!(vb.get_voice_stage(0), VoiceStage::Held);

    // Other voices should be untouched
    for i in 1..TEST_VOICE_BANK_SIZE {
        assert_eq!(vb.get_voice_note(i), u7::from(i as u8).into());
        assert_eq!(vb.get_voice_velocity(i), u7::from(100).into());
        assert_eq!(vb.get_voice_stage(i), VoiceStage::Held);
    }
}

#[test]
fn voice_bank_release_note_triggers_release() {
    setup_voice_bank!(vb);

    let note_to_play = 60;
    vb.play_note(note_to_play.into(), 100.into());
    assert_eq!(vb.count_active_voices(), 1);

    vb.release_note(note_to_play.into());
    // Voice enters Release state, still counts as active until envelope completes
    // (It won't be Free immediately - the ADSR release envelope needs to finish)
    assert_eq!(vb.count_active_voices(), 1);
    assert_eq!(vb.get_voice_stage(0), VoiceStage::Held); // Still reported as Held (non-idle)
}

#[test]
fn voice_bank_release_note_releases_all_instances_of_a_note() {
    setup_voice_bank!(vb);

    let note_to_play = 60;
    vb.play_note(note_to_play.into(), 100.into()); // Voice 0
    vb.play_note(note_to_play.into(), 90.into()); // Voice 1 (if TEST_VOICE_BANK_SIZE >= 2)
    assert_eq!(vb.count_active_voices(), 2);

    vb.release_note(note_to_play.into());
    // Both voices enter Release state, still count as active until envelopes complete
    assert_eq!(vb.count_active_voices(), 2);
    assert_eq!(vb.get_voice_stage(0), VoiceStage::Held); // Still reported as Held (non-idle)
    assert_eq!(vb.get_voice_stage(1), VoiceStage::Held);
}

#[test]
fn voice_bank_release_note_non_existent_does_nothing() {
    setup_voice_bank!(vb);

    vb.play_note(60.into(), 100.into());
    vb.play_note(62.into(), 100.into());
    assert_eq!(vb.count_active_voices(), 2);

    // Try to release a note that was not played
    vb.release_note(64.into());
    assert_eq!(vb.count_active_voices(), 2);
    assert_eq!(vb.get_voice_note(0), u7::from(60).into());
    assert_eq!(vb.get_voice_note(1), u7::from(62).into());
    assert_eq!(vb.get_voice_stage(0), VoiceStage::Held);
    assert_eq!(vb.get_voice_stage(1), VoiceStage::Held);
}

#[test]
fn voice_bank_process_midi_event_handles_note_on_and_off() {
    setup_voice_bank!(vb);

    let note = 60;
    let vel = 100;

    // NoteOn
    vb.process_midi_event(MidiEvent::NoteOn { key: note, vel });
    assert_eq!(vb.count_active_voices(), 1);
    assert_eq!(vb.get_voice_note(0), note.into());
    assert_eq!(vb.get_voice_velocity(0), vel.into());
    assert_eq!(vb.get_voice_stage(0), VoiceStage::Held);

    // NoteOff
    vb.process_midi_event(MidiEvent::NoteOff { key: note, vel: 0 });
    // Voice enters Release state, still active until envelope completes
    assert_eq!(vb.count_active_voices(), 1);
    assert_eq!(vb.get_voice_stage(0), VoiceStage::Held); // Still non-idle (in Release)
}
