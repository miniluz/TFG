#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

= Análisis
<sec_análisis>

Análisis de la competencia, por qué elegí estos requisitos.

== Requisitos
<sec_requisitos>

=== Requisitos funcionales
<sec_rf>

#req("rf_midi_usb", "F")[MIDI USB][
  El sintetizador ha de ser configurable en compilación para conectarse a MIDI por USB.]
#req("rf_midi_wire", "F")[MIDI cable][
  El sintetizador ha de ser configurable en compilación para conectarse a MIDI por cable.]
#req("rf_audio_usb", "F")[Audio USB][
  El sintetizador ha de ser configurable en compilación para conectarse al audio por USB.]
#req("rf_waveforms", "F")[Generación de ondas][
  El sintetizador ha de poder generar ondas sinusoidales, cuadradas, de diente de sierra y triangulares.]
#req("rf_adsr", "F")[ADSR][
  El sintetizador ha de usar un envelope ADSR para obtener audio musical.]
#req("rf_eq", "F")[Ecualización][
  El sintetizador ha de ser configurable en compilación para poder ecualizar la señal.]
#req("rf_polyphony", "F")[Polifonía][
  El sintetizador ha de tener una cantidad de voces configurable en compilación.]
#req("rf_multi_device", "F")[Multiples dispositivos][
  El sintetizador ha de poder ser instalado en al menos dos dispositivos empotrados distintos.]
#req("rf_add_devices", "F")[Añadir dispositivos][
  Debe estar documentado cómo configurar el sintetizador para un nuevo dispositivo empotrado.]
#req("rf_runtime_configuration", "F")[Configuración en ejecución][
  El sintetizador ha de ser configurable en ejecución con elementos físicos conectados a la placa.]

=== Requisitos no funcionales
<sec_rnf>

#req("rnf_speed", "NF")[Velocidad][
  El sintetizador ha de acabar de producir cada bloque de audio antes de que el siguiente se solicite.]
#req("rnf_reliability", "NF")[Fiabilidad][
  El sintetizador ha de operar continuamente sin necesitar un reinicio con uso normal.]
#req("rnf_audio_quality", "NF")[Calidad de audio][
  El sintetizador ha de producir audio libre de distorsiones audibles con uso normal.]
#req("rnf_tests", "NF")[Pruebas][
  El sintetizador ha de tener pruebas que validen su funcionalidad ejecutables en CI.
]

== Análisis de los requisitos
<sec_análisis_de_los_requisitos>

== Lenguaje
<sec_lenguaje>

Los requisitos funcionales pueden ser cumplidos en bastantes lenguajes. El uso de un dispositivo empotrado según el
@rf_multi_device los limita un poco, pero los más usados para el desarrollo empotrado hoy en día son C, C++, Zig, Ada y
Rust.

Los requisitos no funcionales son los que más limitan la elección de lenguaje. El lenguaje más usado para el desarrollo
empotrado es, sin duda, C. El @rnf_speed urge la integración de CMSIS-DSP, una librería de C para realizar operaciones
DSP aprovechando las operaciones del CPU operaciones. Usar C, C++ o Zig permitiría integrarse con la librería sin
problemas. Sin embargo, aunque he usado C en el pasado, no considero que tenga suficiente experiencia para garantizar
que no hayan problemas en memoria o un uso incorrecto accidental del hardware, para cumplir el @rnf_reliability. Un
argumento similar aplica a C++ y a Zig.

Si lo que se busca es garantizar el @rnf_reliability, Ada o incluso SPARK serían una buenas opciones. SPARK es un
lenguaje que permite realizar verificación formal de los programas. Sin embargo, tienen ecosistemas muy pequeños, en
particular con relación al audio, por lo que se tendría que implementar mucha lógica desde cero. La fricción de esta
opción no permitiría realizar el proyecto en el tiempo evaluado.

Rust consigue dar garantías de fiabilidad suficientes para el proyecto y proveer un ecosistema suficiente para el
proyecto. Usando el _borrow checker_, garantiza la seguridad de memoria en tiempo de ejecución. Además, un programa que
no usa código `unsafe` nunca tiene comportamiento indefinido. Las librerías de `HAL` (_hardware abstraction layer_) en
el ecosistema de Rust están construidas con una API que hacen imposible configurar incorrectamente el hardware. Esto
garantiza que la única condición que bloquea el dispositivo sería un bloqueo mutuo. Si se diseña una arquitectura en la
que esto no pueda ocurrir, se garantiza el @rnf_reliability.

// TODO! Explicar borrow checker

Rust también facilita crear código que es compilable al dispositivo empotrado (sin acceso a la librería estándar,
`no_std`) y realizar pruebas para él que sí que tienen acceso a `std` y que son ejecutables en un ordenador `x86_64`,
permitiendo cumplir el @rnf_tests.

También tiene un ecosistema empotrado amplio independiente del hardware, facilitando el uso de librerías para la lectura
de MIDI, las operaciones de coma fija, etc. Finalmente, Rust es el lenguaje con el que tengo más experiencia de los
evaluados, aunque sea principalmente con aplicaciones web. Por todo esto, he elegido usar Rust.

== Ecosistema
<sec_ecosistema>

El desarrollo de aplicaciones complicadas para sistemas empotrados generalmente se realiza mediante el uso de sistemas
operativos de tiempo real (RTOS). Son sistemas operativos generalmente diseñados para microcontroladores que mantienen
consistencia en la cantidad de tiempo que toma aceptar y completar una tarea. Rust permite integrarse con FreeRTOS, un
sistema operativo de código abierto muy usado.

Estos sistemas tienen un modelo de concurrencia apropiativo: el sistema operativo quita control a las tareas para
distribuir el tiempo de ejecución entre ellas. Sin embargo este cambio de tareas conlleva un coste. Se ha de reservar
espacio para poder guardar la pila de cada tarea de manera conservadora. Además, cada vez que hay un cambio de contexto,
se han de guardar todos los registros del CPU a memoria y restaurar el estado de la nueva tarea, además de actualizar
las estructuras de datos que permiten una distribución homogénea.

La alternativa es usar una distribución de tareas cooperativa. En ellos, cada tarea cede el control, generalmente cuando
están bloqueados por una operación I/O, están inactivos, o están esperando una interrupción del CPU. Estas tareas
generalmente se implementan usando máquinas de estado, que indican explícitamente los datos que hay que preservar entre
llamadas. Esto ahorra las reservas de espacio dimensionadas para toda la pila de los RTOS. Además, ahorra almacenar y
restaurar los registros con cada cambio de tarea, ya que este cambio efectivamente es retornar desde una función y
llamar a otra. Añadir tareas es prácticamente gratis en comparación a un RTOS. Sin embargo, puede resultar en que una
tarea que nunca rinda el control detenga el programa. Además, en lenguajes como C, generalmente las máquinas de estado
de las tareas son implementadas manualmente, haciendo que funciones complejas sean menos legibles.

Rust, sin embargo, permite convertir funciones asíncronas en máquinas de estado apropiadas para plataformas empotradas
automáticamente usando Embassy. Esta es la opción más usada por el ecosistema de desarrollo empotrado. Embassy provee un
ejecutor asíncrono mínimo y la capacidad de programar con concurrencia cooperativa usando la sintaxis `async` y `await`,
de manera similar a aplicaciones web en Rust o JavaScript, el área donde tengo más experiencia. Este modelo es
compatible con el sintetizador, ya que consiste en una tarea intensiva para la CPU (la generación de audio) y muchas
tareas cortas restringidas por I/O (botones, MIDI, USB, audio). Además, el ecosistema de Embassy es el más maduro en
Rust, y provee integración con los HALs, USB, y las primitivas de sincronización (canales, señales). Todo esto hace que
Embassy sea la opción más apropiada para el proyecto.

// TODO! Añadir fuentes
// TODO! Por qué Nix

Explicar por qué Rust, por qué Embassy, por qué Nix.
