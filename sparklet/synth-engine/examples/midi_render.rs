use clap::Parser;
use cmsis_rust::CmsisRustOperations as Ops;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use hound::{WavSpec, WavWriter};
use midi::MidiEvent;
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};
use std::fs;
use std::path::PathBuf;
use synth_engine::wavetable::triangle_wavetable::TRIANGLE_WAVETABLE;
use synth_engine::wavetable::{
    saw_wavetable::SAW_WAVETABLE, sine_wavetable::SINE_WAVETABLE,
    square_wavetable::SQUARE_WAVETABLE,
};
use synth_engine::{Q15, SAMPLE_RATE, SynthEngine, WINDOW_SIZE};

const CHANNEL_SIZE: usize = 256;

#[derive(Parser, Debug)]
#[command(name = "MIDI Renderer")]
#[command(about = "Renders MIDI files using the synth engine", long_about = None)]
struct Args {
    /// Wavetable to use (sine, square, sawtooth, triangle)
    #[arg(long)]
    wavetable: String,

    /// Attack config (0-255)
    #[arg(long)]
    attack: u8,

    /// Decay/Release config (0-255)
    #[arg(long, name = "decay-release")]
    decay_release: u8,

    /// Sustain config (0-255)
    #[arg(long)]
    sustain: u8,

    /// Number of voices
    #[arg(long)]
    voices: usize,

    /// Path to MIDI file
    #[arg(long, default_value = "./The Entertainer.mid")]
    midi: PathBuf,

    /// Output WAV file path
    #[arg(long, default_value = "./test-results/output.wav")]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Select wavetable
    let wavetable: &[Q15; 256] = match args.wavetable.to_lowercase().as_str() {
        "sine" => &SINE_WAVETABLE,
        "square" => &SQUARE_WAVETABLE,
        "sawtooth" | "saw" => &SAW_WAVETABLE,
        "triangle" => &TRIANGLE_WAVETABLE,
        _ => {
            eprintln!("Invalid wavetable. Choose: sine, square, sawtooth or triangle");
            std::process::exit(1);
        }
    };

    println!(
        "Rendering MIDI file with {} wavetable, attack={}, decay_release={}, sustain={}, voices={}",
        args.wavetable, args.attack, args.decay_release, args.sustain, args.voices
    );

    // Read MIDI file
    let midi_data = fs::read(&args.midi).expect("Failed to read MIDI file");
    let smf = Smf::parse(&midi_data).expect("Failed to parse MIDI file");

    // Extract timing information
    let ticks_per_beat = match smf.header.timing {
        midly::Timing::Metrical(tpb) => tpb.as_int() as u64,
        _ => {
            eprintln!("SMPTE timing not supported");
            std::process::exit(1);
        }
    };

    println!(
        "MIDI file: {} tracks, {} ticks per beat",
        smf.tracks.len(),
        ticks_per_beat
    );

    // Parse all MIDI events and convert to sample times
    let mut events = parse_midi_events(&smf, ticks_per_beat);

    // Sort events by time
    events.sort_by_key(|(time, _)| *time);

    println!("Parsed {} MIDI events", events.len());

    // Calculate total render duration (last event + 5 seconds margin)
    let last_event_time = events.last().map(|(time, _)| *time).unwrap_or(0);
    let margin_samples = SAMPLE_RATE as u64 * 15;
    let total_samples = last_event_time + margin_samples;
    let duration_secs = total_samples as f64 / SAMPLE_RATE as f64;

    println!(
        "Total duration: {:.2} seconds ({} samples)",
        duration_secs, total_samples
    );

    // Render audio based on voice count
    let audio_samples = match args.voices {
        2 => render_audio::<2>(
            &events,
            total_samples,
            wavetable,
            args.attack,
            args.decay_release,
            args.sustain,
        ),
        4 => render_audio::<4>(
            &events,
            total_samples,
            wavetable,
            args.attack,
            args.decay_release,
            args.sustain,
        ),
        16 => render_audio::<16>(
            &events,
            total_samples,
            wavetable,
            args.attack,
            args.decay_release,
            args.sustain,
        ),
        _ => {
            eprintln!("Invalid voice count. Choose: 2, 4, or 16");
            std::process::exit(1);
        }
    };

    println!("Rendered {} samples", audio_samples.len());

    // Write WAV file
    write_wav(&args.output, &audio_samples);

    println!("Output written to: {}", args.output.display());
}

fn parse_midi_events(smf: &Smf, ticks_per_beat: u64) -> Vec<(u64, MidiEvent)> {
    let mut events = Vec::new();
    let mut tempo_us_per_qn = 500_000u64; // Default: 120 BPM

    for track in &smf.tracks {
        let mut accumulated_ticks = 0u64;
        let mut accumulated_samples = 0u64;
        let mut last_tempo_us = tempo_us_per_qn;

        for event in track {
            accumulated_ticks += event.delta.as_int() as u64;

            match event.kind {
                TrackEventKind::Meta(MetaMessage::Tempo(tempo)) => {
                    // Convert accumulated ticks since last tempo change to samples
                    let samples_per_tick =
                        (SAMPLE_RATE as u64 * last_tempo_us) / (ticks_per_beat * 1_000_000);
                    accumulated_samples += accumulated_ticks * samples_per_tick;
                    accumulated_ticks = 0;

                    // Update tempo
                    tempo_us_per_qn = tempo.as_int() as u64;
                    last_tempo_us = tempo_us_per_qn;
                }
                TrackEventKind::Midi {
                    channel: _,
                    message,
                } => {
                    // Convert current position to samples
                    let samples_per_tick =
                        (SAMPLE_RATE as u64 * last_tempo_us) / (ticks_per_beat * 1_000_000);
                    let sample_time = accumulated_samples + (accumulated_ticks * samples_per_tick);

                    match message {
                        MidiMessage::NoteOn { key, vel } => {
                            if vel.as_int() > 0 {
                                events.push((
                                    sample_time,
                                    MidiEvent::NoteOn {
                                        key: key.as_int(),
                                        vel: vel.as_int(),
                                    },
                                ));
                            } else {
                                // Note on with velocity 0 is note off
                                events.push((
                                    sample_time,
                                    MidiEvent::NoteOff {
                                        key: key.as_int(),
                                        vel: 0,
                                    },
                                ));
                            }
                        }
                        MidiMessage::NoteOff { key, vel } => {
                            events.push((
                                sample_time,
                                MidiEvent::NoteOff {
                                    key: key.as_int(),
                                    vel: vel.as_int(),
                                },
                            ));
                        }
                        _ => {} // Ignore other MIDI messages
                    }
                }
                _ => {} // Ignore other event types
            }
        }
    }

    events
}

fn render_audio<const VOICE_COUNT: usize>(
    events: &[(u64, MidiEvent)],
    total_samples: u64,
    wavetable: &'static [Q15; 256],
    attack: u8,
    decay_release: u8,
    sustain: u8,
) -> Vec<i16> {
    // Create channel for MIDI events
    let channel = Channel::<NoopRawMutex, MidiEvent, CHANNEL_SIZE>::new();
    let sender = channel.sender();
    let receiver = channel.receiver();

    // Create synth engine
    let mut synth_engine = SynthEngine::<'_, '_, _, CHANNEL_SIZE, VOICE_COUNT, WINDOW_SIZE>::new(
        receiver,
        wavetable,
        sustain,
        attack,
        decay_release,
    );

    // Prepare output buffer
    let mut output = Vec::with_capacity(total_samples as usize);
    let mut current_sample = 0u64;
    let mut event_index = 0;

    // Rendering loop
    while current_sample < total_samples {
        // Send all events that should happen before the next window
        while event_index < events.len() && events[event_index].0 <= current_sample {
            let (_, event) = events[event_index];
            sender.try_send(event).ok(); // Ignore if channel is full
            event_index += 1;
        }

        // Render one window of samples
        let mut buffer = [Q15::ZERO; WINDOW_SIZE];
        synth_engine.render_samples::<Ops>(&mut buffer);

        // Convert Q15 to i16 and add to output
        for sample in &buffer {
            if (current_sample as usize) < total_samples as usize {
                output.push(sample.to_bits());
                current_sample += 1;
            }
        }
    }

    output
}

fn write_wav(path: &PathBuf, samples: &[i16]) {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create output directory");
    }

    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec).expect("Failed to create WAV file");

    for &sample in samples {
        writer.write_sample(sample).expect("Failed to write sample");
    }

    writer.finalize().expect("Failed to finalize WAV file");
}
