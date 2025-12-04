use super::*;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use pretty_assertions::assert_eq;
use wavetable::sine_wavetable::SINE_WAVETABLE;

// Test CMSIS implementation
type TestOps = cmsis_rust::CmsisRustOperations;

// Constants for test voice bank and channel sizes
const TEST_VOICE_BANK_SIZE: usize = 4;
const TEST_CHANNEL_SIZE: usize = 16; // Larger channel to avoid overflow in tests

// Default ADSR config for tests (moderate settings)
const TEST_SUSTAIN: u8 = 200;
const TEST_ATTACK: u8 = 50;
const TEST_DECAY_RELEASE: u8 = 100;

// Fast ADSR config for lifecycle tests
const FAST_SUSTAIN: u8 = 200;
const FAST_ATTACK: u8 = 10;
const FAST_DECAY_RELEASE: u8 = 10;

// Macro for easily setting up a SynthEngine instance with a Sender
macro_rules! setup_synth_engine {
    ($sender:ident, $se:ident) => {
        let channel = Channel::<NoopRawMutex, MidiEvent, TEST_CHANNEL_SIZE>::new();
        let $sender = channel.sender();
        let receiver = channel.receiver();
        let mut $se = SynthEngine::<'_, '_, _, _, TEST_VOICE_BANK_SIZE, WINDOW_SIZE>::new(
            receiver,
            &SINE_WAVETABLE,
            TEST_SUSTAIN,
            TEST_ATTACK,
            TEST_DECAY_RELEASE,
        );
    };
}

// --- Property-Based Tests ---

#[test]
fn test_silence_when_no_notes() {
    setup_synth_engine!(_sender, se);

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    se.render_samples::<TestOps>(&mut buffer);

    // Property: Output should be all zeros when no notes are playing
    assert!(
        buffer.iter().all(|&s| s == Q15::ZERO),
        "Output should be silent with no active voices"
    );
}

#[test]
fn test_activity_when_playing() {
    setup_synth_engine!(sender, se);

    // Send a NoteOn
    sender
        .try_send(MidiEvent::NoteOn { key: 60, vel: 100 })
        .unwrap();

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    se.render_samples::<TestOps>(&mut buffer);

    // Property: Output should be non-zero when a note is playing (at least during attack/sustain)
    let non_zero_count = buffer.iter().filter(|&&s| s != Q15::ZERO).count();
    assert!(
        non_zero_count > 0,
        "Output should contain non-zero samples when a note is playing"
    );
}

#[test]
fn test_voice_allocation_correctness() {
    setup_synth_engine!(sender, se);

    // Send 3 NoteOn events (less than max voices)
    sender
        .try_send(MidiEvent::NoteOn { key: 60, vel: 100 })
        .unwrap();
    sender
        .try_send(MidiEvent::NoteOn { key: 62, vel: 100 })
        .unwrap();
    sender
        .try_send(MidiEvent::NoteOn { key: 64, vel: 100 })
        .unwrap();

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    se.render_samples::<TestOps>(&mut buffer);

    // Property: Correct number of voices should be active
    assert_eq!(
        se.get_voice_bank().count_active_voices(),
        3,
        "Should have 3 active voices"
    );
}

#[test]
fn test_voice_stealing_via_queue() {
    setup_synth_engine!(sender, se);

    // Fill all voices (4) and then add one more
    sender
        .try_send(MidiEvent::NoteOn { key: 60, vel: 100 })
        .unwrap();
    sender
        .try_send(MidiEvent::NoteOn { key: 62, vel: 100 })
        .unwrap();
    sender
        .try_send(MidiEvent::NoteOn { key: 64, vel: 100 })
        .unwrap();
    sender
        .try_send(MidiEvent::NoteOn { key: 65, vel: 100 })
        .unwrap();
    sender
        .try_send(MidiEvent::NoteOn { key: 67, vel: 100 })
        .unwrap(); // 5th note - should queue

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    se.render_samples::<TestOps>(&mut buffer);

    // Property: Should still have at most TEST_VOICE_BANK_SIZE voices
    assert!(
        se.get_voice_bank().count_active_voices() <= TEST_VOICE_BANK_SIZE,
        "Should not exceed max voice count"
    );

    // Render a few more times to let the queued note get assigned
    for _ in 0..10 {
        se.render_samples::<TestOps>(&mut buffer);
    }

    // Eventually all voices should be active again
    assert_eq!(
        se.get_voice_bank().count_active_voices(),
        TEST_VOICE_BANK_SIZE,
        "All voices should be active after queue processing"
    );
}

#[test]
fn test_envelope_lifecycle_to_idle() {
    // Use fast ADSR for this test
    let channel = Channel::<NoopRawMutex, MidiEvent, TEST_CHANNEL_SIZE>::new();
    let sender = channel.sender();
    let receiver = channel.receiver();
    let mut se = SynthEngine::<'_, '_, _, _, TEST_VOICE_BANK_SIZE, WINDOW_SIZE>::new(
        receiver,
        &SINE_WAVETABLE,
        FAST_SUSTAIN,
        FAST_ATTACK,
        FAST_DECAY_RELEASE,
    );

    // Send NoteOn then immediate NoteOff
    sender
        .try_send(MidiEvent::NoteOn { key: 60, vel: 100 })
        .unwrap();
    sender
        .try_send(MidiEvent::NoteOff { key: 60, vel: 0 })
        .unwrap();

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];

    // Render many times to let the envelope reach idle
    // Need enough cycles for attack + release to complete
    for _ in 0..100 {
        se.render_samples::<TestOps>(&mut buffer);
    }

    // Property: Voice should eventually return to idle after enough time
    assert_eq!(
        se.get_voice_bank().count_active_voices(),
        0,
        "Voice should eventually reach idle state after NoteOff"
    );

    // And output should be silent again
    se.render_samples::<TestOps>(&mut buffer);
    assert!(
        buffer.iter().all(|&s| s == Q15::ZERO),
        "Output should be silent after all voices idle"
    );
}

#[test]
fn test_velocity_scaling() {
    setup_synth_engine!(sender1, se1);
    setup_synth_engine!(sender2, se2);

    // Send same note with different velocities
    sender1
        .try_send(MidiEvent::NoteOn { key: 69, vel: 127 })
        .unwrap();
    sender2
        .try_send(MidiEvent::NoteOn { key: 69, vel: 64 })
        .unwrap();

    let mut buffer1 = [Q15::ZERO; WINDOW_SIZE];
    let mut buffer2 = [Q15::ZERO; WINDOW_SIZE];

    // Render multiple cycles to let the ADSR envelope progress
    for _ in 0..10 {
        se1.render_samples::<TestOps>(&mut buffer1);
        se2.render_samples::<TestOps>(&mut buffer2);
    }

    // Property: Higher velocity should produce larger absolute amplitude (on average)
    let avg_abs_1: f64 = buffer1
        .iter()
        .map(|&s| s.abs().to_num::<f64>())
        .sum::<f64>()
        / WINDOW_SIZE as f64;
    let avg_abs_2: f64 = buffer2
        .iter()
        .map(|&s| s.abs().to_num::<f64>())
        .sum::<f64>()
        / WINDOW_SIZE as f64;

    assert!(
        avg_abs_1 > avg_abs_2,
        "Velocity 127 should produce larger amplitude than velocity 64 (avg {} vs {})",
        avg_abs_1,
        avg_abs_2
    );
}

#[test]
fn test_queue_overflow_handling() {
    setup_synth_engine!(sender, se);

    // Send more notes than the queue can hold (queue size = voice bank size = 4)
    // Fill all voices first
    for i in 0..TEST_VOICE_BANK_SIZE {
        sender
            .try_send(MidiEvent::NoteOn {
                key: 60 + i as u8,
                vel: 100,
            })
            .unwrap();
    }

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    se.render_samples::<TestOps>(&mut buffer);

    // Now overflow the queue with many more notes
    for i in 0..10 {
        sender
            .try_send(MidiEvent::NoteOn {
                key: 70 + i as u8,
                vel: 100,
            })
            .unwrap();
    }

    // Property: Should not crash and should handle gracefully
    for _ in 0..20 {
        se.render_samples::<TestOps>(&mut buffer);
    }

    // Should still have voices active
    assert!(
        se.get_voice_bank().count_active_voices() > 0,
        "Should still have active voices after queue overflow"
    );
}

#[test]
fn test_rapid_note_on_off_sequences() {
    // Use fast ADSR for this test
    let channel = Channel::<NoopRawMutex, MidiEvent, TEST_CHANNEL_SIZE>::new();
    let sender = channel.sender();
    let receiver = channel.receiver();
    let mut se = SynthEngine::<'_, '_, _, _, TEST_VOICE_BANK_SIZE, WINDOW_SIZE>::new(
        receiver,
        &SINE_WAVETABLE,
        FAST_SUSTAIN,
        FAST_ATTACK,
        FAST_DECAY_RELEASE,
    );

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];

    // Rapidly alternate NoteOn/NoteOff
    for _ in 0..20 {
        sender
            .try_send(MidiEvent::NoteOn { key: 60, vel: 100 })
            .unwrap();
        sender
            .try_send(MidiEvent::NoteOff { key: 60, vel: 0 })
            .unwrap();
        se.render_samples::<TestOps>(&mut buffer);
    }

    // Property: Should not crash and eventually return to silence
    for _ in 0..50 {
        se.render_samples::<TestOps>(&mut buffer);
    }

    assert_eq!(
        se.get_voice_bank().count_active_voices(),
        0,
        "All voices should be idle after rapid on/off sequence"
    );
}

#[test]
fn test_note_off_releases_correct_voice() {
    setup_synth_engine!(sender, se);

    // Play two notes
    sender
        .try_send(MidiEvent::NoteOn { key: 60, vel: 100 })
        .unwrap();
    sender
        .try_send(MidiEvent::NoteOn { key: 64, vel: 100 })
        .unwrap();

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    se.render_samples::<TestOps>(&mut buffer);

    assert_eq!(se.get_voice_bank().count_active_voices(), 2);

    // Release first note
    sender
        .try_send(MidiEvent::NoteOff { key: 60, vel: 0 })
        .unwrap();
    se.render_samples::<TestOps>(&mut buffer);

    // Should trigger release for note 60, but note 64 should still be held/sustaining
    // Note: count_active_voices counts non-idle, so releasing voice is still active until idle
    assert!(
        se.get_voice_bank().count_active_voices() > 0,
        "At least one voice should still be active (note 64)"
    );

    // After many renders, note 60 should be idle but 64 should still be sustained
    for _ in 0..50 {
        se.render_samples::<TestOps>(&mut buffer);
    }

    // Note 64 should still be active (sustained)
    assert_eq!(
        se.get_voice_bank().count_active_voices(),
        1,
        "Note 64 should still be sustained"
    );
}

#[test]
#[should_panic]
fn test_wrong_buffer_size_panics() {
    setup_synth_engine!(_sender, se);

    let mut buffer = [Q15::ZERO; 64]; // Wrong size!
    se.render_samples::<TestOps>(&mut buffer);
}

#[test]
fn test_voice_stealing_doesnt_release_all_voices() {
    setup_synth_engine!(sender, se);

    // Fill all 4 voices
    for i in 0..TEST_VOICE_BANK_SIZE {
        sender
            .try_send(MidiEvent::NoteOn {
                key: 60 + i as u8,
                vel: 100,
            })
            .unwrap();
    }

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    se.render_samples::<TestOps>(&mut buffer);

    // All voices should be active
    assert_eq!(se.get_voice_bank().count_active_voices(), TEST_VOICE_BANK_SIZE);

    // Play one more note over the limit
    sender
        .try_send(MidiEvent::NoteOn { key: 70, vel: 100 })
        .unwrap();

    // Render once - this will trigger quick_release on one voice
    se.render_samples::<TestOps>(&mut buffer);

    // Immediately after first render, check that we haven't released all voices
    // At most 1 voice should be in quick release or have become idle
    let quick_release_after_first = se.get_voice_bank().count_voices_in_quick_release();
    assert!(
        quick_release_after_first <= 1,
        "Should have at most 1 voice in quick release, got {}",
        quick_release_after_first
    );

    // Render a few more cycles
    for _ in 0..10 {
        se.render_samples::<TestOps>(&mut buffer);
    }

    // Property: The new note should have been allocated, and we should still have active voices
    // This proves voice stealing worked without releasing all voices
    assert_eq!(
        se.get_voice_bank().count_active_voices(),
        TEST_VOICE_BANK_SIZE,
        "Should still have all voices active after voice stealing"
    );
}

#[test]
fn test_multiple_queued_notes_release_appropriate_voices() {
    setup_synth_engine!(sender, se);

    // Fill all 4 voices
    for i in 0..TEST_VOICE_BANK_SIZE {
        sender
            .try_send(MidiEvent::NoteOn {
                key: 60 + i as u8,
                vel: 100,
            })
            .unwrap();
    }

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    se.render_samples::<TestOps>(&mut buffer);

    // Queue 3 more notes over the limit
    for i in 0..3 {
        sender
            .try_send(MidiEvent::NoteOn {
                key: 70 + i as u8,
                vel: 100,
            })
            .unwrap();
    }

    // Render several cycles
    for _ in 0..5 {
        se.render_samples::<TestOps>(&mut buffer);
    }

    // Property: At most 3 voices should be in quick release (not all 4)
    let quick_release_count = se.get_voice_bank().count_voices_in_quick_release();
    assert!(
        quick_release_count <= 3,
        "Should have at most 3 voices in quick release for 3 queued notes, got {}",
        quick_release_count
    );

    // Property: Should not release all voices
    assert!(
        quick_release_count < TEST_VOICE_BANK_SIZE,
        "Should not release all {} voices",
        TEST_VOICE_BANK_SIZE
    );
}

#[test]
fn test_quick_release_not_repeated_per_cycle() {
    setup_synth_engine!(sender, se);

    // Fill all 4 voices with notes that will stay active
    for i in 0..TEST_VOICE_BANK_SIZE {
        sender
            .try_send(MidiEvent::NoteOn {
                key: 60 + i as u8,
                vel: 100,
            })
            .unwrap();
    }

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    se.render_samples::<TestOps>(&mut buffer);
    assert_eq!(se.get_voice_bank().count_active_voices(), TEST_VOICE_BANK_SIZE);

    // Queue multiple notes over the limit
    sender
        .try_send(MidiEvent::NoteOn { key: 70, vel: 100 })
        .unwrap();
    sender
        .try_send(MidiEvent::NoteOn { key: 72, vel: 100 })
        .unwrap();

    // Render once - should trigger quick_release on voices as needed
    se.render_samples::<TestOps>(&mut buffer);
    let first_count = se.get_voice_bank().count_voices_in_quick_release();

    // Render several more times
    for _ in 0..5 {
        se.render_samples::<TestOps>(&mut buffer);
    }

    let final_count = se.get_voice_bank().count_voices_in_quick_release();

    // Property: We shouldn't have released all voices
    // The key insight is that we should have at most as many voices in quick release
    // as we have queued notes (2), not all 4 voices
    assert!(
        first_count <= 2,
        "Should have released at most 2 voices initially for 2 queued notes, got {}",
        first_count
    );

    assert!(
        final_count < TEST_VOICE_BANK_SIZE,
        "Should not have all {} voices in quick release, got {}",
        TEST_VOICE_BANK_SIZE,
        final_count
    );

    // Eventually all voices should be active with the new notes
    for _ in 0..10 {
        se.render_samples::<TestOps>(&mut buffer);
    }

    assert_eq!(
        se.get_voice_bank().count_active_voices(),
        TEST_VOICE_BANK_SIZE,
        "Should have all voices active after queue processing"
    );
}
