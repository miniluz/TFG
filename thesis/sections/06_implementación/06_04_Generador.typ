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

=== Pruebas

/* TODO */
