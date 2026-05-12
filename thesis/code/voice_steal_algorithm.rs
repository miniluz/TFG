while let Ok(event) = self.receiver.try_receive() {
    match event {
        MidiEvent::NoteOff { key, vel: _ } => {
            self.voice_bank.release_note(key.into());
            self.note_queue
                .retain(|PendingNote { note, velocity: _ }| note.as_u8() != key);
        }
        MidiEvent::NoteOn { key, vel } => {
            let pending = PendingNote {
                note: key.into(),
                velocity: vel.into(),
            };
            // Add, dropping oldest
            if self
                .note_queue
                .iter()
                .all(|PendingNote { note, velocity: _ }| note.as_u8() != key)
            {
                let _ = self.note_queue.push_back(pending);
            }
        }
    }
}

while let Some(&pending) = self.note_queue.front() {
    match self.voice_bank.play_note(pending.note, pending.velocity) {
        PlayNoteResult::Success => {
            self.note_queue.pop_front();
        }
        PlayNoteResult::AllVoicesBusy => {
            let queue_count = self.note_queue.len();
            let quick_release_count = self.voice_bank.count_voices_in_quick_release();

            for _ in 0..(queue_count - quick_release_count) {
                self.voice_bank.quick_release();
            }

            break;
        }
    }
}
