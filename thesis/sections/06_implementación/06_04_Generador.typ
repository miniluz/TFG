#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Generador

El generador es un componente simple que combina el ADSR, los osciladores y el banco de voces. Recibe un canal que actúa
como una cola de eventos MIDI pendientes. Su método principal es `render_samples`, que recibe un buffer de longitud `L`
y lo llena de las siguientes $L$ muestras siguiendo los siguientes pasos:

+ Procesa los eventos MIDI que le han llegado desde el último `render_samples`, como se explica en la
  @sec_detalles_algoritmo_voice_bank.
+ Por cada voz:
  + Calcula las muestras que genera el oscilador para ese periodo.
  + Calcula las muestras del envolvente ADSR para ese periodo.
  + Las multiplica, así aplicando la envolvente ADSR a la amplitud de la onda que genera el oscilador.
  + Las divide para acomodar el número de voces total (en concreto, hace el mínimo _bit shift_ a la derecha para que se
    divida entre al menos la cantidad de voces $|V|$).
  + La suma al buffer de salida, que se inicializa a cero.

=== Conexión con la salida de audio

La salida de audio funciona por muestreo. Cuando el controlador de la salida recibe una solicitud de transmición de un
bloque de audio, ha de responder de forma casi inmediata. Debido a ésto, la generación se ejecuta en otra tarea, y
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
