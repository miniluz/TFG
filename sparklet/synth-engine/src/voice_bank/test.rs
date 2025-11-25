use super::*;
use crate::wavetable::sine_wavetable::SINE_WAVETABLE;
use midi::u7;
use pretty_assertions::assert_eq;

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
        let _ = vb.play_note((i as u8).into(), 100.into());
        assert_eq!(vb.count_active_voices(), i + 1);
        assert_eq!(vb.get_voice_note(i), u7::from(i as u8).into());
        assert_eq!(vb.get_voice_velocity(i), u7::from(100).into());
        assert_eq!(vb.get_voice_stage(i), VoiceStage::Held);
    }
    assert_eq!(vb.count_active_voices(), TEST_VOICE_BANK_SIZE);
}

#[test]
fn voice_bank_play_note_returns_err_when_full() {
    setup_voice_bank!(vb);

    // Fill all voices
    for i in 0..TEST_VOICE_BANK_SIZE {
        let result = vb.play_note((i as u8).into(), 100.into());
        assert_eq!(result, Ok(()));
    }
    assert_eq!(vb.count_active_voices(), TEST_VOICE_BANK_SIZE);

    // Try to play one more note, it should return Err(()) since all voices are busy
    let extra_note = TEST_VOICE_BANK_SIZE as u8;
    let result = vb.play_note(extra_note.into(), 120.into());
    assert_eq!(result, Err(()));

    // All voices should be untouched
    assert_eq!(vb.count_active_voices(), TEST_VOICE_BANK_SIZE); // Still full
    for i in 0..TEST_VOICE_BANK_SIZE {
        assert_eq!(vb.get_voice_note(i), u7::from(i as u8).into());
        assert_eq!(vb.get_voice_velocity(i), u7::from(100).into());
        assert_eq!(vb.get_voice_stage(i), VoiceStage::Held);
    }
}

#[test]
fn voice_bank_release_note_triggers_release() {
    setup_voice_bank!(vb);

    let note_to_play = 60;
    let _ = vb.play_note(note_to_play.into(), 100.into());
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
    let result1 = vb.play_note(note_to_play.into(), 100.into()); // Voice 0
    assert_eq!(result1, Ok(()));
    let result2 = vb.play_duplicate_note(note_to_play.into(), 90.into()); // Voice 1 (if TEST_VOICE_BANK_SIZE >= 2)
    assert_eq!(result2, Ok(()));
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

    let _ = vb.play_note(60.into(), 100.into());
    let _ = vb.play_note(62.into(), 100.into());
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

#[test]
fn voice_bank_quick_release_selects_quietest_in_release() {
    setup_voice_bank!(vb);

    // Fill all 4 voices and advance them to sustain
    use cmsis_interface::Q15;
    let mut buffer = [Q15::ZERO; 128];

    for i in 0..TEST_VOICE_BANK_SIZE {
        vb.play_note((60 + i as u8).into(), 100.into()).unwrap();
        // Advance to sustain stage
        for _ in 0..10 {
            vb.voices[i].adsr.get_samples::<128>(&mut buffer);
        }
    }

    // Release all voices so they enter Release state
    for i in 0..TEST_VOICE_BANK_SIZE {
        vb.release_note((60 + i as u8).into());
    }

    // Verify all are in Release
    for voice in &vb.voices {
        assert!(voice.adsr.is_in_release());
    }

    // Run a few samples on first voice to make it quieter than the others
    vb.voices[0].adsr.get_samples::<128>(&mut buffer);
    vb.voices[0].adsr.get_samples::<128>(&mut buffer);

    // Verify voice 0 is quietest
    assert!(vb.voices[0].adsr.output < vb.voices[1].adsr.output);
    assert!(vb.voices[0].adsr.output < vb.voices[2].adsr.output);
    assert!(vb.voices[0].adsr.output < vb.voices[3].adsr.output);

    // Call quick_release - it should select voice 0 (quietest in Release)
    vb.quick_release();

    // Verify voice 0 is now in QuickRelease
    assert!(vb.voices[0].adsr.is_in_quick_release());

    // Verify other voices are still in Release
    for i in 1..TEST_VOICE_BANK_SIZE {
        assert!(vb.voices[i].adsr.is_in_release());
    }
}

#[test]
fn voice_bank_quick_release_selects_oldest_when_none_in_release() {
    setup_voice_bank!(vb);

    // Fill all 4 voices with different timestamps
    for i in 0..TEST_VOICE_BANK_SIZE {
        vb.play_note((60 + i as u8).into(), 100.into()).unwrap();
    }

    // All voices are in Attack/Decay/Sustain (none in Release)
    // Voice 0 should have the oldest timestamp
    assert_eq!(vb.voices[0].timestamp, 1);
    assert_eq!(vb.voices[1].timestamp, 2);
    assert_eq!(vb.voices[2].timestamp, 3);
    assert_eq!(vb.voices[3].timestamp, 4);

    // Call quick_release - should select oldest voice (voice 0)
    vb.quick_release();

    // Verify voice 0 is in QuickRelease
    assert!(vb.voices[0].adsr.is_in_quick_release());

    // Verify other voices are not in QuickRelease
    for i in 1..TEST_VOICE_BANK_SIZE {
        assert!(!vb.voices[i].adsr.is_in_quick_release());
    }
}

#[test]
fn voice_bank_quick_release_handles_all_idle() {
    setup_voice_bank!(vb);

    // All voices are idle initially
    assert_eq!(vb.count_active_voices(), 0);

    // Call quick_release on empty voice bank - should be a no-op
    vb.quick_release();

    // Verify all voices are still idle
    assert_eq!(vb.count_active_voices(), 0);
    for voice in &vb.voices {
        assert!(voice.adsr.is_idle());
    }
}
