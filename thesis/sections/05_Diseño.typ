#import "@preview/deal-us-tfc-template:1.0.0": *

= Diseño
<sec_diseño>

== Arquitectura
<sec_arquitectura>

#figure(
  image("/figures/Diagrama de la arquitectura.drawio.pdf", width: 70%),
  caption: "Diagrama de la arquitectura de Sparklet",
)<fig_diagrama_arquitectura>

La arquitectura de Sparklet se puede ver en el @fig_diagrama_arquitectura. Consiste en 5 grupos de tareas: el motor de
síntesis, el gestor de configuración, las entradas hardware de configuración, la entrada MIDI, y la salida de audio,
conectados con primitivas asíncronas.

La salida de audio es muestreada por el sistema periódicamente (en USB, cada $1000 "Hz"$). Esto garantiza la
sincronización de la frecuencia de muestreo, y evita tener que mantener un reloj interno y sincronizarlo con el del
ordenador. Cuando Sparklet recibe un muestreo, tiene que dar una respuesta casi inmediata, por lo que el bloque de audio
tiene que estar listo para que únicamente tenga que ser transmitido. Debido a esto, decidí extraer la generación del
bloque de audio a otra tarea que lo prepara de antemano: el motor de síntesis. Son conectadas con una cola de 2 bloques
de audio completos con _back-pressure_, lo que significa que la cola bloquea al escritor hasta que hay un espacio libre.
Cuando se envía el bloque, se consume el mensaje y libera un espacio, despertando el motor de síntesis para que genera
el siguiente bloque.

El motor de audio recibe una cola de eventos MIDI que han llegado desde la generación del último bloques de audio. Esta
cola la llena la tarea de entrada MIDI, que lee un _stream_ de bytes que procesa y convierte en eventos MIDI,
descartando si Sparklet no soporta el evento o si la cola está llena. El primer paso en la generación de audio es el
procesamiento de estos eventos MIDI. Es necesario que lo realice el motor de audio, ya que el banco de voces necesita de
información del volumen de cada nota para poder decidir cuáles dejar de tocar.

La configuración, sin embargo, no depende del estado del motor de síntesis, por lo que se extrae a una tarea separada de
menor prioridad. La tarea de configuración lee el hardware usando polling, por defecto cada $5 "ms"$, y procesa con el
gestor de configuración los datos. Cada cierto tiempo, por defecto cada $100 "ms"$, envía la nueva configuración por un
triple buffer al motor de síntesis.

== Rendimiento

// TODO: análisis del rendimiento mejor caso
// TODO: medir el rendimiento en otro chip

=== Instrucciones DSP
<sec_inst_dsp>

#figure(
  image("/figures/CMSIS Interface.drawio.pdf", width: 50%),
  caption: "Diagrama del uso de CMSIS Interface",
)<fig_cmsis_interface>

Para poder realizar los cálculos necesarios con la velocidad suficiente, es necesario aprovechar las instrucciones del
hardware. Para esto, se usa la librería CMSIS-DSP. Sin embargo, esta librería usa instrucciones de ensamblador que no
están disponibles en `x86_64`, la arquitectura de la computadora de desarrollo, sino tan solo en ARM Cortex M7. Para
poder ejecutar los mismos módulos tanto en el chip como en la computadora, las operaciones necesarias se abstraen detrás
de una interfaz: CMSIS Interface.

Hay dos implementaciones de esta interfaz, como se indica en la @fig_cmsis_interface. Una, CMSIS Rust, usa Rust puro y
se puede compilar a `x86_64`, y se provee a los módulos en las pruebas automáticas. La otra, CMSIS Native, usa las
funciones de la librería CMSIS-DSP, y se provee en la ejecución.

Tanto CMSIS Rust como CMSIS Native son probadas por la misma batería de pruebas, para garantizar que sus
implementaciones son idénticas. Se usan macros definidas en el módulo CMSIS Interface para generar las pruebas de ambas
implementaciones, garantizando que son iguales. Las pruebas de CMSIS Rust se pueden ejecutar automáticamente en
`x86_64`, pero las de CMSIS Native han de ser ejecutadas en el chip manualmente cada vez que se añade una función a la
interfaz.
