#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Motor de síntesis

El motor de síntesis es un componente simple que integra el generador con el banco de filtros. Es la capa exterior del
sistema de audio, pero realmente es muy simple. Únicamente provee una interfaz sencilla que inicializa todos sus
componentes y abstrae su funcionamiento.

En el archivo `sparklet/synth-engine/examples/midi_render.rs` se puede encontrar una prueba que calcula las muestras
para un archivo MIDI del dominio público (The Entertainer de Scott Joplin) y las guarda en un archivo de audio con
formato WAV. `sparklet/synth-engine/render_all.sh` la ejecuta con varias configuraciones para ver cómo estas afectan al
audio. Por ejemplo, una vez ejecutado, en `sparklet/synth-engine/test-results/entertainer_sawtooth_mid_4v.wav` se puede
ver cómo se escucharía con usando la onda de diente de sierra con cuatro voces y un ataque, decaimiento y relajación
bajas.
