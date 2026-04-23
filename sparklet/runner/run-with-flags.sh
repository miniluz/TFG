#!/usr/bin/env bash

if [ $# -eq 0 ]; then
   echo "Usage: $0 <command> [args...]" >&2
   exit 1
fi

midi=$(tomlq '.connections.midi' Config.toml)
audio=$(tomlq '.connections.audio' Config.toml)
octave=$(tomlq '.features["octave_filter"]' Config.toml)
config=$(tomlq '.features["configurable"]' Config.toml)

flags=(--no-default-features)

[ "$midi" = '"usb"' ] && flags+=(--features midi-usb)
[ "$midi" = '"din"' ] && flags+=(--features midi-din)
[ "$audio" = '"usb"' ] && flags+=(--features audio-usb)
[ "$octave" = "true" ] && flags+=(--features octave-filter)
[ "$config" = "true" ] && flags+=(--features configurable)

DEFMT_LOG="${DEFMT_LOG:-off}" exec "$@" "${flags[@]}"
