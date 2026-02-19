use clap::Parser;
use cmsis_rust::CmsisRustOperations as Ops;
use config::{Config, ConfigEvent, ConfigManager};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel, signal::Signal};
use hound::{WavSpec, WavWriter};
use midi::MidiEvent;
use rand::{RngExt, SeedableRng};
use std::f64::consts::PI;
use std::path::PathBuf;
use synth_engine::{Q15, SAMPLE_RATE, SynthEngine, WINDOW_SIZE};

const CHANNEL_SIZE: usize = 256;
const PAGE_AMOUNT: usize = 4; // Pages 0-1: ADSR/oscillator, Pages 2-3: Octave filter
const ENCODER_AMOUNT: usize = 3;
const OCTAVE_FILTER_FIRST_PAGE: usize = 2;

#[derive(Parser, Debug)]
#[command(name = "Config Oscillation Demo")]
#[command(about = "Demonstrates dynamic config updates with oscillating parameters", long_about = None)]
struct Args {
    #[arg(long, default_value = "60")]
    duration: u32,

    #[arg(long, default_value = "4")]
    voices: usize,

    #[arg(long, default_value = "1234")]
    seed: u64,

    #[arg(long, default_value = "./test-results/config_oscillation.wav")]
    output: PathBuf,
}

fn oscillate(time_sec: f64, period_sec: f64, phase_offset: f64) -> u8 {
    let normalized = ((time_sec / period_sec + phase_offset) * 2.0 * PI).sin();
    let scaled = (normalized + 1.0) / 2.0; // Map from [-1, 1] to [0, 1]
    (scaled * 255.0) as u8
}

fn render_audio<const VOICE_COUNT: usize>(duration_sec: u32, seed: u64) -> Vec<i16> {
    let channel = Channel::<NoopRawMutex, MidiEvent, CHANNEL_SIZE>::new();
    let sender = channel.sender();
    let receiver = channel.receiver();

    let config_signal = Signal::<NoopRawMutex, Config<PAGE_AMOUNT, ENCODER_AMOUNT>>::new();
    let mut config_manager = ConfigManager::new(&config_signal);

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
        OCTAVE_FILTER_FIRST_PAGE,
    >::new(receiver, &config_signal);

    let total_samples = duration_sec as usize * SAMPLE_RATE as usize;
    let mut output = Vec::with_capacity(total_samples);

    let mut settings = [[(0., 0.); ENCODER_AMOUNT]; PAGE_AMOUNT];
    let mut last_values = [[0; ENCODER_AMOUNT]; PAGE_AMOUNT];

    let mut rng_seed = [0; 32];
    rng_seed[0..8].clone_from_slice(&seed.to_ne_bytes());
    let mut rng = rand::rngs::StdRng::from_seed(rng_seed);

    for page in settings.iter_mut() {
        for encoder in page.iter_mut() {
            *encoder = (rng.random_range(2.0..7.0), rng.random_range(0.0..1.0))
        }
    }

    let note_pattern = [
        24, 28, 31, 36, 40, 43, 48, 52, 55, 60, 64, 67, 72, 76, 79, 84,
    ]; // C, E, G, C
    let len = note_pattern.len();

    let mut note_index: usize = 0;
    let samples_per_note = SAMPLE_RATE as usize / 8; // 4 notes per second
    let mut samples_since_note = 0;

    println!(
        "Rendering {} seconds of audio with oscillating config...",
        duration_sec
    );

    let mut buffer = [Q15::ZERO; WINDOW_SIZE];
    let mut current_sample = 0usize;

    while current_sample < total_samples {
        let time_sec = current_sample as f64 / SAMPLE_RATE as f64;

        for (page, (settings, last_values)) in
            settings.iter().zip(last_values.iter_mut()).enumerate()
        {
            for (encoder, ((period, offset), last_value)) in
                settings.iter().zip(last_values.iter_mut()).enumerate()
            {
                let delta = if page != 0 && encoder != 0 {
                    let target = oscillate(time_sec, *period, *offset);

                    (target as i16 - (*last_value) as i16).signum() as i8
                } else {
                    let target_osc_type: u8 = ((time_sec / 10.0).floor()) as u8 % 4;

                    target_osc_type as i8 - *last_value
                };

                if delta != 0 {
                    config_manager.handle_event(ConfigEvent::EncoderChange {
                        encoder: encoder as u8,
                        amount: delta,
                    });
                    *last_value = (*last_value).saturating_add(delta);
                }
            }
            config_manager.handle_event(ConfigEvent::PageChange { amount: 1 });
        }

        // Play notes in pattern
        if samples_since_note >= samples_per_note {
            let note = note_pattern[note_index];
            sender
                .try_send(MidiEvent::NoteOn {
                    key: note,
                    vel: 127,
                })
                .ok();

            let prev_note = note_pattern[note_index.wrapping_sub(1).rem_euclid(len)];
            sender
                .try_send(MidiEvent::NoteOff {
                    key: prev_note,
                    vel: 0,
                })
                .ok();

            note_index = (note_index + 1) % len;
            samples_since_note = 0;
        }
        samples_since_note += WINDOW_SIZE;

        synth_engine.render_samples::<Ops>(&mut buffer);

        let samples_to_copy = (total_samples - current_sample).min(WINDOW_SIZE);
        output.extend_from_slice(bytemuck::cast_slice::<Q15, i16>(&buffer[..samples_to_copy]));

        current_sample += samples_to_copy;

        if current_sample.is_multiple_of(SAMPLE_RATE as usize * 5) {
            println!("Progress: {:.1}s / {}s", time_sec, duration_sec,);
        }
    }

    for note in note_pattern {
        sender
            .try_send(MidiEvent::NoteOff { key: note, vel: 0 })
            .ok();
    }

    println!("Rendered {} samples", output.len());
    output
}

fn main() {
    let args = Args::parse();

    println!("Config Oscillation Demo");
    println!("Duration: {} seconds", args.duration);
    println!("Voices: {}", args.voices);
    println!("Seed: {}", args.seed);
    println!();

    let samples = match args.voices {
        1 => render_audio::<1>(args.duration, args.seed),
        2 => render_audio::<2>(args.duration, args.seed),
        4 => render_audio::<4>(args.duration, args.seed),
        8 => render_audio::<8>(args.duration, args.seed),
        16 => render_audio::<16>(args.duration, args.seed),
        _ => panic!("Unsupported voice count: {}", args.voices),
    };

    if let Some(parent) = args.output.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create output directory");
    }

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
