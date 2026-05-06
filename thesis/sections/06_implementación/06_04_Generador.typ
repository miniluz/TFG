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

=== Conexión con la salida de audio

La salida de audio funciona por sondeo. Cuando el controlador de ésta recibe la solicitud de un bloque, ha de responder
de forma casi inmediata. Debido a ésto, la generación opera en otra tarea y genera los siguientes dos bloques de audio
de antemano. De esta manera, para transmitirlos basta con copiarlos, pues todos los cálculos se realizan de antemano.

Se usan dos bloques ya que cada uno conlleva un retraso, puesto que se calculan de antemano y se mantiene lleno el
buffer: si se sondea una vez cada milisegundo, entonces se responde con un bloque generado hace dos milisegundos, con
las notas que estaban siendo tocadas en ese momento, no las actuales.

Los bloques que calcula el generador se transmiten a la tarea de salida de audio con un
`embassy_sync::zerocopy_channel`. El canal tiene _back-pressure_: cuando está lleno, enviar un mensaje espera a que haya
un espacio disponible. Así se sincroniza la tarea de generación de audio con la de envío: la generación de un bloque
nuevo espera a que la salida de audio consuma el último.

/*
El `zerocopy_channel` no copia los datos internamente, pero a cambio requiere que el usuario reserve un espacio para
enviar y marque los datos como enviados cuando acabe. En este caso, como se está enviando una cantidad de datos no
trivial, la eficiencia vale la complejidad.
*/


=== Pruebas

/* TODO */
