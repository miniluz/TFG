use clap::Parser;
use cmsis_rust::CmsisRustOperations as Ops;
use config::{Config, ConfigEvent, ConfigManager};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel, signal::Signal};
use hound::{WavSpec, WavWriter};
use midi::MidiEvent;
use std::f64::consts::PI;
use std::path::PathBuf;
use synth_engine::{Q15, SAMPLE_RATE, SynthEngine, WINDOW_SIZE};

const CHANNEL_SIZE: usize = 256;
const PAGE_AMOUNT: usize = 2;
const ENCODER_AMOUNT: usize = 3;

#[derive(Parser, Debug)]
#[command(name = "Config Oscillation Demo")]
#[command(about = "Demonstrates dynamic config updates with oscillating parameters", long_about = None)]
struct Args {
    /// Duration in seconds
    #[arg(long, default_value = "60")]
    duration: u32,

    /// Number of voices
    #[arg(long, default_value = "4")]
    voices: usize,

    /// Output WAV file path
    #[arg(long, default_value = "./test-results/config_oscillation.wav")]
    output: PathBuf,
}

/// Generate an oscillating value between 0 and 255
fn oscillate(time_sec: f64, period_sec: f64, phase_offset: f64) -> u8 {
    let normalized = ((time_sec / period_sec + phase_offset) * 2.0 * PI).sin();
    let scaled = (normalized + 1.0) / 2.0; // Map from [-1, 1] to [0, 1]
    (scaled * 255.0) as u8
}

fn render_audio<const VOICE_COUNT: usize>(
    duration_sec: u32,
) -> Vec<i16> {
    // Create channel for MIDI events
    let channel = Channel::<NoopRawMutex, MidiEvent, CHANNEL_SIZE>::new();
    let sender = channel.sender();
    let receiver = channel.receiver();

    // Create config signal and manager
    let config_signal = Signal::<NoopRawMutex, Config<PAGE_AMOUNT, ENCODER_AMOUNT>>::new();
    let mut config_manager = ConfigManager::new(&config_signal);

    // Create synth engine
    let mut synth_engine = SynthEngine::<
        '_,
        '_,
        '_,
        NoopRawMutex,
        CHANNEL_SIZE,
        VOICE_COUNT,
        WINDOW_SIZE,
        PAGE_AMOUNT,
        ENCODER_AMOUNT,
    >::new(
        receiver,
        &config_signal,
    );

    let total_samples = duration_sec as usize * SAMPLE_RATE as usize;
    let mut output = Vec::with_capacity(total_samples);

    // Oscillation parameters (different periods and phase offsets for each parameter)
    let attack_period = 5.3;
    let attack_offset = 0.0;

    let sustain_period = 7.1;
    let sustain_offset = 0.25;

    let release_period = 6.2;
    let release_offset = 0.5;

    // Start playing a continuous pattern of notes
    let note_pattern = [60, 64, 67, 72]; // C, E, G, C (one octave up)
    let mut note_index = 0;
    let samples_per_note = SAMPLE_RATE as usize / 4; // 4 notes per second
    let mut samples_since_note = 0;

    // Track last encoder values to detect changes
    let mut last_attack = 127u8;
    let mut last_sustain = 127u8;
    let mut last_release = 127u8;
    let mut last_osc_type = 0u8;

    println!("Rendering {} seconds of audio with oscillating config...", duration_sec);
    println!("Attack oscillation: {} second period, offset {}", attack_period, attack_offset);
    println!("Sustain oscillation: {} second period, offset {}", sustain_period, sustain_offset);
    println!("Release oscillation: {} second period, offset {}", release_period, release_offset);
    println!("Oscillator switches every 10 seconds");

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    let mut current_sample = 0usize;

    while current_sample < total_samples {
        let time_sec = current_sample as f64 / SAMPLE_RATE as f64;

        // Calculate target values for this moment in time
        let target_attack = oscillate(time_sec, attack_period, attack_offset);
        let target_sustain = oscillate(time_sec, sustain_period, sustain_offset);
        let target_release = oscillate(time_sec, release_period, release_offset);

        // Oscillator type increments every 10 seconds
        let target_osc_type = ((time_sec / 10.0).floor() as u8) % 4;

        // Generate encoder change events to reach target values
        // We'll send incremental changes each frame to simulate smooth encoder turns
        if target_attack != last_attack {
            let delta = (target_attack as i16 - last_attack as i16).signum() as i8;
            config_manager.handle_event(ConfigEvent::EncoderChange {
                encoder: 0,
                amount: delta,
            });
            last_attack = (last_attack as i16 + delta as i16).clamp(0, 255) as u8;
        }

        if target_sustain != last_sustain {
            let delta = (target_sustain as i16 - last_sustain as i16).signum() as i8;
            config_manager.handle_event(ConfigEvent::EncoderChange {
                encoder: 1,
                amount: delta,
            });
            last_sustain = (last_sustain as i16 + delta as i16).clamp(0, 255) as u8;
        }

        if target_release != last_release {
            let delta = (target_release as i16 - last_release as i16).signum() as i8;
            config_manager.handle_event(ConfigEvent::EncoderChange {
                encoder: 2,
                amount: delta,
            });
            last_release = (last_release as i16 + delta as i16).clamp(0, 255) as u8;
        }

        // Switch to the next page and change oscillator type
        if target_osc_type != last_osc_type {
            // First, switch to page 1
            config_manager.handle_event(ConfigEvent::PageChange { amount: 1 });

            // Change oscillator
            let delta = (target_osc_type as i16 - last_osc_type as i16).signum() as i8;
            config_manager.handle_event(ConfigEvent::EncoderChange {
                encoder: 0,
                amount: delta,
            });
            last_osc_type = (last_osc_type as i16 + delta as i16).clamp(0, 3) as u8;

            // Switch back to page 0
            config_manager.handle_event(ConfigEvent::PageChange { amount: -1 });

            let osc_name = match target_osc_type {
                0 => "Sine",
                1 => "Sawtooth",
                2 => "Square",
                _ => "Triangle",
            };
            println!("Switching to {} waveform at {:.1}s", osc_name, time_sec);
        }

        // Play notes in pattern
        if samples_since_note >= samples_per_note {
            let note = note_pattern[note_index];
            sender.try_send(MidiEvent::NoteOn { key: note, vel: 100 }).ok();

            // Release previous note
            if note_index > 0 {
                let prev_note = note_pattern[note_index - 1];
                sender.try_send(MidiEvent::NoteOff { key: prev_note, vel: 0 }).ok();
            }

            note_index = (note_index + 1) % note_pattern.len();
            samples_since_note = 0;
        }
        samples_since_note += WINDOW_SIZE;

        // Render audio
        synth_engine.render_samples::<Ops>(&mut buffer);

        // Convert and append to output
        let samples_to_copy = (total_samples - current_sample).min(WINDOW_SIZE);
        output.extend_from_slice(
            &bytemuck::cast_slice::<Q15, i16>(&buffer[..samples_to_copy])
        );

        current_sample += samples_to_copy;

        // Progress indicator
        if current_sample % (SAMPLE_RATE as usize * 5) == 0 {
            println!("Progress: {:.1}s / {}s (Attack={}, Sustain={}, Release={}, Osc={})",
                time_sec, duration_sec, last_attack, last_sustain, last_release, last_osc_type);
        }
    }

    // Release all notes
    for note in note_pattern {
        sender.try_send(MidiEvent::NoteOff { key: note, vel: 0 }).ok();
    }

    println!("Rendered {} samples", output.len());
    output
}

fn main() {
    let args = Args::parse();

    println!("Config Oscillation Demo");
    println!("Duration: {} seconds", args.duration);
    println!("Voices: {}", args.voices);
    println!();

    let samples = match args.voices {
        1 => render_audio::<1>(args.duration),
        2 => render_audio::<2>(args.duration),
        4 => render_audio::<4>(args.duration),
        8 => render_audio::<8>(args.duration),
        16 => render_audio::<16>(args.duration),
        _ => panic!("Unsupported voice count: {}", args.voices),
    };

    // Ensure output directory exists
    if let Some(parent) = args.output.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create output directory");
    }

    // Write WAV file
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(&args.output, spec).expect("Failed to create WAV writer");

    for sample in samples {
        writer.write_sample(sample).expect("Failed to write sample");
    }

    writer.finalize().expect("Failed to finalize WAV file");

    println!("Output written to: {}", args.output.display());
}
