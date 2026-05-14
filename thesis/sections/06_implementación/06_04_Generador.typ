#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Generador

El generador encapsula los componentes que calculan la señal de audio, antes de su procesamiento posterior por los
efectos como el ecualizador. Coordina los osciladores, el ADSR y el banco de voces. Su método principal es
`render_samples`, que recibe un buffer de longitud `L` y lo llena de las siguientes $L$ muestras siguiendo los
siguientes pasos:

+ Hace que el `VoiceBank` procese los eventos MIDI que le han llegado desde el último `render_samples`, como se explica
  en la @sec_procesado_midi.
+ Por cada voz:
  + Calcula las muestras que genera el oscilador para ese periodo.
  + Calcula las muestras del envolvente ADSR para ese periodo.
  + Las multiplica, así aplicando la envolvente ADSR a la amplitud de la onda que genera el oscilador.
  + Las divide para acomodar el número de voces total (en concreto, hace el mínimo _bit shift_ a la derecha para que se
    divida entre al menos la cantidad de voces $|V|$).
  + La suma al buffer de salida, que se inicializa a cero.
