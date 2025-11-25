#!/usr/bin/env bash

# Shell script to render all 27 combinations of wavetables, ADSR settings, and voice counts

set -e

# Configuration
MIDI_FILE="./The Entertainer.mid"
OUTPUT_DIR="./test-results"
SUSTAIN=200

# ADSR configurations
declare -A ATTACK
ATTACK[fast]=20
ATTACK[mid]=40
ATTACK[slow]=235

declare -A DECAY_RELEASE
DECAY_RELEASE[fast]=20
DECAY_RELEASE[mid]=127
DECAY_RELEASE[slow]=235

# Arrays for iteration
WAVETABLES=("sine" "square" "sawtooth" "triangle")
ADSR_MODES=("fast" "mid" "slow")
VOICE_COUNTS=(2 4 16)

echo "Starting MIDI rendering for all 27 combinations..."
echo "MIDI file: $MIDI_FILE"
echo "Output directory: $OUTPUT_DIR"
echo ""

total=0
for wavetable in "${WAVETABLES[@]}"; do
    for adsr in "${ADSR_MODES[@]}"; do
        for voices in "${VOICE_COUNTS[@]}"; do
            total=$((total + 1))
            output_file="${OUTPUT_DIR}/entertainer_${wavetable}_${adsr}_${voices}v.wav"

            echo "[$total/36] Rendering: $wavetable, $adsr ADSR, $voices voices"

            cargo run --release --example midi_render -- \
                --wavetable "$wavetable" \
                --attack "${ATTACK[$adsr]}" \
                --decay-release "${DECAY_RELEASE[$adsr]}" \
                --sustain "$SUSTAIN" \
                --voices "$voices" \
                --midi "$MIDI_FILE" \
                --output "$output_file"

            echo "  ✓ Created: $output_file"
            echo ""
        done
    done
done

echo "All 27 combinations rendered successfully!"
echo "Output files are in: $OUTPUT_DIR"
