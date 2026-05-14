#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Motor de síntesis

El motor de síntesis es un componente simple que integra el generador con los efectos, como el ecualizador. Es la capa
exterior del sistema de audio. Proporciona una interfaz sencilla que inicializa todos sus componentes y abstrae su
funcionamiento.

=== Prueba extremo a extremo

En el archivo `sparklet/synth-engine/examples/midi_render.rs` se puede encontrar una prueba que usa el motor de síntesis
para calcular las muestras de audio para un archivo MIDI del dominio público (The Entertainer de Scott Joplin) y las
guarda en un archivo con formato WAV. `sparklet/synth-engine/render_all.sh` la ejecuta con varias configuraciones para
ver cómo estas afectan al audio. Una vez ejecutado, habrá un archivo de audio por permutación de la configuración. Por
ejemplo, en `sparklet/synth-engine/test-results/entertainer_sawtooth_mid_16v.wav` se podrá escuchar cómo se comporta el
motor de síntesis configurado con la onda de diente de sierra, dieciséis voces y ataque, decaimiento y relajación
intermedios.

=== Conexión con la salida de audio

La salida de audio funciona por muestreo. Cuando el controlador de la salida recibe una solicitud de transmición de un
bloque de audio, ha de responder de forma casi inmediata. Debido a esto, la generación se ejecuta en otra tarea, y
genera los siguientes dos bloques de antemano. De esta manera, para transmitirlos basta con copiarlos: no es necesario
esperar a que se calculen. Se usan dos bloques ya que cada uno conlleva un retraso, puesto que se calculan de antemano y
se mantiene lleno el buffer: si se muestrea una vez cada milisegundo, entonces se responde con un bloque generado hace
dos milisegundos con las notas que estaban siendo tocadas en ese entonces, en lugar de las actuales.

Los bloques que calcula el generador se transmiten a la tarea de salida de audio con un
`embassy_sync::zerocopy_channel`. Este canal se usa para sincronizar la tarea de generación de audio con la de envío. La
salida de audio consume el mensaje al acabar de transmitirlo. Para poder enviar un bloque, la generación de audio tiene
que esperar a que haya un espacio libre en el canal, por lo que espera a que el mensaje haya sido transmitido. Se dice
que un canal donde la operación de enviar espera a que haya un espacio libre tiene _back-pressure_.

El `zerocopy_channel` no copia los datos internamente al transmitir un mensaje, lo que lo hace más eficiente para
mensajes grandes como bloques de audio. En su lugar, se ha de reservar una posición tanto para escribir como para leer,
e indicar manualmente cuándo se ha terminado de usarla. En este caso, esto es una ventaja, pues permite que la
generación ocurra cuando se haya acabado de transmitir el bloque por USB en lugar de cuando se haya recibido.
