#import "@preview/deal-us-tfc-template:1.0.0": *

= Diseño
<sec_diseño>

== Arquitectura
<sec_arquitectura>

#figure(
  image("/figures/Diagrama de la arquitectura.drawio.pdf", width: 70%),
  caption: "Diagrama de la arquitectura de Sparklet",
)<fig_diagrama_arquitectura>

La arquitectura de Sparklet se puede ver en el @fig_diagrama_arquitectura. Consiste en 4 grupos de tareas: el motor de
síntesis, el gestor de configuración, la entrada MIDI, y la salida de audio, conectados con primitivas asíncronas.

La salida de audio es muestreada por el sistema periódicamente (en USB, cada $1000 "Hz"$ @ref_web_usb_audio). Esto
garantiza la sincronización de la frecuencia de muestreo, y evita tener que mantener un reloj interno y sincronizarlo
con el del ordenador. Cuando Sparklet recibe un muestreo, tiene responder de forma casi inmediata, por lo que el bloque
de audio tiene que ser calculado de antemano. Debido a esto, la generación de los bloques de audio se realiza en otra
tarea: el motor de síntesis. Son conectadas con una cola de 2 bloques de audio completos con _back-pressure_, lo que
implica que la cola bloquea al escritor (el motor de síntesis) hasta que hay un espacio libre (se envía un bloque de
audio). Una vez enviado el bloque, se marca como consumido, lo que despierta el motor de síntesis para que genere el
siguiente.

El motor de audio tiene una cola de eventos MIDI que han llegado desde la generación del último bloque de audio. La
tarea de entrada MIDI lee un _stream_ de bytes y añade a la cola los eventos MIDI que hay en él, descartándolos si
Sparklet no los soporta o si la cola está llena.

Idealmente se procesarían los eventos MIDI para saber qué notas se están tocando actualmente en su propia tarea, en
lugar de en la de generación de audio. Sin embargo, el algoritmo que determina qué nota soltar cuando tocar una nueva
nota supera el límite de polifonía requiere de información privada al motor de síntesis.

La configuración, sin embargo, no depende de información privada, por lo que se extrae a una tarea separada de menor
prioridad. La tarea de configuración lee el hardware usando polling, por defecto cada $5 "ms"$, y procesa con el gestor
de configuración los datos. Cada cierto tiempo, por defecto cada $100 "ms"$, envía la nueva configuración por un triple
buffer al motor de síntesis.

== Rendimiento

// TODO: análisis del rendimiento mejor caso
// TODO: medir el rendimiento en otro chip

=== Instrucciones DSP
<sec_inst_dsp>

#figure(
  image("/figures/CMSIS Interface.drawio.pdf", width: 50%),
  caption: "Diagrama del uso de CMSIS Interface",
  placement: auto,
)<fig_cmsis_interface>

Para poder realizar los cálculos necesarios con la velocidad suficiente, es necesario aprovechar las instrucciones del
hardware. Para esto, se usa la biblioteca CMSIS-DSP. Sin embargo, esta biblioteca usa instrucciones de ensamblador que no
están disponibles en `x86_64`, la arquitectura de la computadora de desarrollo, sino tan solo en ARM Cortex M7
@ref_web_cmsis_dsp. Para poder ejecutar los mismos módulos tanto en el chip como en la computadora, las operaciones
necesarias se abstraen detrás de una interfaz: CMSIS Interface.

Hay dos implementaciones de esta interfaz, como se indica en la @fig_cmsis_interface. Una, CMSIS Rust, usa Rust puro y
se puede compilar a `x86_64`, y se provee a los módulos en las pruebas automáticas. La otra, CMSIS Native, usa las
funciones de la biblioteca CMSIS-DSP, y se provee en la ejecución.

Tanto CMSIS Rust como CMSIS Native son probadas por la misma batería de pruebas, para garantizar que sus
implementaciones son idénticas. Se usan macros definidas en el módulo CMSIS Interface para generar las pruebas de ambas
implementaciones, garantizando que son iguales. Las pruebas de CMSIS Rust se pueden ejecutar automáticamente en
`x86_64`, pero las de CMSIS Native han de ser ejecutadas en el chip manualmente cada vez que se añade una función a la
interfaz.
