#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Motor de síntesis

El motor de síntesis es un componente simple que integra el generador con el banco de filtros. Es la capa exterior del
sistema de audio. Proporciona una interfaz sencilla que inicializa todos sus componentes y abstrae su funcionamiento.

En el archivo `sparklet/synth-engine/examples/midi_render.rs` se puede encontrar una prueba que calcula las muestras
para un archivo MIDI del dominio público (The Entertainer de Scott Joplin) y las guarda en un archivo de audio con
formato WAV. `sparklet/synth-engine/render_all.sh` la ejecuta con varias configuraciones para ver cómo estas afectan al
audio. Por ejemplo, una vez ejecutado, en `sparklet/synth-engine/test-results/entertainer_sawtooth_mid_16v.wav` se puede
probar cómo se escucharía el motor de síntesis usando la onda de diente de sierra con dieciséis voces y una
configuración de ataque, decaimiento y relajación intermedia.
