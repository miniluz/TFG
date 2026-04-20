#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Generador

El generador es un elemento relativamente simple que combina el ADSR, los osciladores y el banco de voces. Recibe una
cola de eventos MIDI que procesar.

El método principal que tiene es `render_samples`, que recibe un buffer de longitud `L` y lo llena de las siguientes
muestras de la siguiente manera:

+ Procesa los eventos MIDI que le han llegado a la cola desde el último `render_samples`, como se explica en la
  @sec_procesamiento_midi_voice_bank.
+ Por cada voz:
  + Calcula las muestras que genera el oscilador para ese periodo.
  + Calcula las muestras del envolvente ADSR para ese periodo.
  + Las multiplica juntas.
  + Las divide para acomodar el número de voces total (en concreto, hace el mínimo _bit shift_ para que divida entre al
    menos $|V|$).
  + Las acumula al buffer de salida.

== Conexión con la salida de audio

La salida de audio funciona por sondeo. Cuando se solicita la siguiente salida de audio, el controlador tiene que
transmitirlo de forma casi inmediata. Debido a ésto, la generación opera en otra tarea y genera las siguientes dos
salidas de audio de antemano, para dar un margen de error. No se usan más de dos ya que cada salida precalculada añade
retraso: si se sondea una vez cada milisegundo, entonces escucharás la salida que procesó los eventos MIDI y cambios de
configuración con dos segundos de retraso.

El audio que escribe el generador se envía a la tarea de salida de audio usando un `embassy_sync::zerocopy_channel`. El
canal tiene _back-pressure_, es decir que cuando está lleno al intentar enviar esperas a que se consuma el mensaje
actual. De esta manera se sincroniza la tarea de generación de audio con la de envío: la generación de audio espera a
que la salida de audio consuma el último mensaje.

El `zerocopy_channel` no copia los datos internamente, pero a cambio requiere que el usuario reserve un espacio para
enviar y marque los datos como enviados cuando acabe. En este caso, como se está enviando una cantidad de datos no
trivial, la eficiencia vale la complejidad.


=== Pruebas

/* TODO */
