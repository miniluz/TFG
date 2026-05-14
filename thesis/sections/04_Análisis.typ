#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

= Análisis
<sec_análisis>

/* TODO Análisis de la competencia */

== Requisitos
<sec_requisitos>

=== Requisitos funcionales
<sec_rf>

#req("rf_midi_usb", "F")[MIDI USB][
  El sintetizador ha de ser configurable en compilación para conectarse a MIDI por USB.]
#req("rf_midi_din", "F")[MIDI DIN][
  El sintetizador ha de ser configurable en compilación para conectarse a MIDI por un puerto DIN.]
#req("rf_audio_usb", "F")[Audio USB][
  El sintetizador ha de ser configurable en compilación para conectarse al audio por USB.]
#req("rf_ondas", "F")[Generación de ondas][
  El sintetizador ha de poder generar ondas sinusoidales, cuadradas, de diente de sierra y triangulares.]
#req("rf_adsr", "F")[ADSR][
  El sintetizador ha de modular la amplitud de la onda con un envolvente ADSR configurable.]
#req("rf_ecualizador", "F")[Ecualización][
  El sintetizador ha de ser configurable en compilación para poder ecualizar la señal.]
#req("rf_polifonía", "F")[Polifonía][
  El sintetizador ha de tener una cantidad de voces configurable en compilación.]
#req("rf_multi_dispositivos", "F")[Multiples dispositivos][
  El sintetizador ha de poder ser instalado en al menos dos dispositivos empotrados distintos.]
#req("rf_añadir_dispositivos", "F")[Añadir dispositivos][
  Debe estar documentado cómo configurar el sintetizador para un nuevo dispositivo empotrado.]
#req("rf_configuración_ejecución", "F")[Configuración en ejecución][
  El sintetizador ha de ser configurable en ejecución con elementos físicos conectados a la placa.]

=== Requisitos no funcionales
<sec_rnf>

#req("rnf_rendimiento", "NF")[Rendimiento][
  El sintetizador ha de acabar de producir cada bloque de audio antes de que el siguiente se solicite.]
#req("rnf_fiabilidad", "NF")[Fiabilidad][
  El sintetizador ha de operar continuamente sin necesitar un reinicio.]
#req("rnf_calidad_de_audio", "NF")[Calidad de audio][
  El sintetizador ha de producir audio libre de distorsiones perceptibles.]
#req("rnf_pruebas", "NF")[Pruebas][
  El sintetizador ha de tener pruebas que validen su funcionalidad ejecutadas automáticamente durante el desarrollo de
  forma visible.
]

== Análisis de los requisitos
<sec_análisis_de_los_requisitos>

=== Lenguaje de programación
<sec_lenguaje>

En cuanto a la selección del lenguaje, se evaluaron C, C++, Zig, SPARK y Rust, por ser lenguajes populares para el
desarrollo empotrado.

C es una de los lenguajes más populares en el desarrollo empotrado. El @rnf_rendimiento urge la integración de
CMSIS-DSP, una biblioteca de C para realizar operaciones DSP aprovechando las operaciones del CPU operaciones. Usar C,
C++ o Zig permitiría integrarse con la librería sin problemas. Sin embargo, aunque he usado C en el pasado, no considero
que tenga suficiente experiencia para garantizar que no hayan problemas de memoria o un uso incorrecto accidental del
hardware, para cumplir el @rnf_fiabilidad. Un argumento similar aplica a C++ y a Zig.

SPARK es una opción apropiada para conseguir el @rnf_fiabilidad, ya que permite verificar formalmente los programas
@ref_web_ada_formal_proof. Sin embargo, tiene un ecosistema pequeño, en particular en lo que respecta al audio, por lo
que se tendría que implementar mucha lógica desde cero. La fricción de esta opción no permitiría realizar el proyecto a
tiempo.

Rust aporta garantías de fiabilidad suficientes y proporciona un ecosistema suficiente para facilitar la realización del
proyecto. He podido encontrar bibliotecas para las necesidades del proyecto compatibles con el desarrollo empotrado
(gestión de operaciones de coma fija, de MIDI, de USB, ejecución de pruebas, etc.), además de herramientas útiles (para
leer los mensajes del chip, evaluar el uso de memoria del binario, etc.).

En cuanto a la fiabilidad: usando sistemas como el _borrow checker_, ayuda a comprobar la seguridad de memoria en tiempo
de compilación @ref_web_rust_lifetimes. Además, el comportamiento indefinido únicamente puede ocurrir en código `unsafe`
(o código seguro que depende de código `unsafe`) @ref_web_rust_undefined. Es común no escribir código `unsafe` como
parte de tu programa y depender de bibliotecas que lo usan, minimizando el riesgo de que ocurran si las bibliotecas son
revisadas. Las librerías de `HAL` (_hardware abstraction layer_) en el ecosistema de Rust están construidas con una API
diseñada para validar en compilación que la configuración del hardware es correcta. Si se diseña una arquitectura en la
que no puedan ocurrir bloqueos mutuos (_deadlocks_), se puede tener seguridad de que el programa nunca se tendrá que
reiniciar, cumpliendo el @rnf_fiabilidad.

Rust también permite realizar compilación cruzada @ref_web_rust_cross, permitiendo que el mismo código sea compilado
tanto a `x86_64`, la arquitectura del ordenador, como a `thumbv7em` (_ARM Cortex M7_), la arquitectura del
microcontrolador. El proyecto aprovecha esto moviendo todo el código posible a _crates_ (paquetes) que son
independientes del hardware. Esto permite desarrollar sin necesidad de tener el microcontrolador a mano, además de
automatizar las pruebas en workflows de GitHub Actions para que sus resultados sean visibles, para cumplir el
@rnf_pruebas.

Finalmente, Rust es el lenguaje con el que tengo más experiencia de los evaluados. Por todo esto, he elegido usarlo.

=== Ecosistema de desarrollo
<sec_ecosistema>

Las aplicaciones complejas para sistemas empotrados generalmente se realizan usando sistemas operativos de tiempo real
(RTOS). Son sistemas operativos generalmente diseñados para microcontroladores que mantienen consistencia en la cantidad
de tiempo que toma aceptar y completar una tarea @ref_web_rtos. Rust permite integrarse con FreeRTOS, un sistema
operativo de código abierto muy usado @ref_web_rust_freertos.

FreeRTOS, y la mayoría de RTOS, suelen tener un modelo de concurrencia apropiativo: el sistema operativo quita control a
las tareas para distribuir el tiempo de ejecución entre ellas @ref_web_freertos. Sin embargo este cambio de tareas
conlleva un coste. Se ha de reservar espacio para poder guardar la pila de cada tarea de manera conservadora. Además,
cada vez que hay un cambio de contexto, se han de guardar todos los registros del CPU a memoria y restaurar el estado de
la nueva tarea, además de actualizar las estructuras de datos que permiten una distribución homogénea
@ref_web_cooperative_multitasking.

La alternativa es usar una distribución de tareas cooperativa. En ellos, cada tarea cede el control, generalmente cuando
está bloqueada por una operación I/O, está inactiva, o en general está esperando una interrupción del CPU. Estas tareas
generalmente se implementan usando máquinas de estado, que indican explícitamente los datos que hay que preservar entre
llamadas. Esto ahorra las reservas de espacio dimensionadas para toda la pila de los RTOS. Además, ahorra almacenar y
restaurar los registros con cada cambio de tarea, ya que este cambio efectivamente es retornar desde una función y
llamar a otra. Añadir tareas en un sistema cooperativo es muy eficiente en comparación a un RTOS
@ref_web_cooperative_multitasking. Sin embargo, puede resultar en que una tarea que nunca rinda el control detenga el
programa. La multitarea cooperativa es apropiada para Sparklet, ya que el sintetizador consiste en una única tarea
intensiva para la CPU, la generación de audio, que se ejecuta por muestreo, además de tareas ligeras para el CPU
restringidas por I/O (hardware, MIDI).

Un inconveniente es que, en lenguajes como C, generalmente las máquinas de estado de las tareas son implementadas
manualmente, haciendo que funciones complejas sean menos legibles. Este método se puede ver en el
@cod_maquina_estado_manual. En Rust, estas máquinas de estado pueden ser creadas usando las funciones asíncronas, usando
una sintaxis que parece secuencial con `async` y `await`, de manera similar al desarrollo web. Este método se puede ver
en el @cod_maquina_estado_async. Estas funciones son transformadas en máquinas de estados automáticamente, que
implementa la interfaz `Future`. Embassy proporciona un ejecutor cooperativo ligero para plataformas empotradas basada
en los `Future` de Rust.
#figure(
  grid(
    columns: 1,
    inset: 0.5em,
    [
      #figure(
        ```rust
        enum State { A, B }

        fn step(state: &mut State, ready: bool) {
            match state {
                State::A => {
                    println!("A");
                    *state = State::B;
                }

                State::B => {
                    if ready {
                        println!("B");
                        *state = State::A;
                    }
                }
            }
        }
        ```,
        caption: [Una tarea cooperativa que alterna entre el estado A y B, implementada a mano como una máquina de
          estados.
        ],
      )<cod_maquina_estado_manual>
    ],
    [
      #figure(
        ```rust
        async fn task() {
          loop {
              println!("A");
              wait().await;
              println!("B");
          }
        }
        ```,
        caption: [Una tarea cooperativa que alterna entre el estado A y B, implementada con la sintaxis `async` y
          convertida en una máquina de estados por el compilador.],
      )
      <cod_maquina_estado_async>
    ],
  ),
  numbering: none,
  placement: auto,
)

Debido a la popularidad de Embassy, su ecosistema es bastante maduro. Ofrece _hardware abstraction layers_, APIs de Rust
que abstraen las características del hardware (p. ej. entrada, salida, _pull-ups_). Usan el sistema de tipos de Rust
para garantizar que los estados inválidos del hardware generan fallos durante la compilación (en lugar de durante la
ejecución), ayudando a conseguir el @rnf_fiabilidad. Por ejemplo, es imposible activar las interrupciones en los pines
`PA5` y `PB5` simultáneamente, ya que crear una entrada con interrupciones consume un `struct` `EXTI5` del que
únicamente se puede obtener uno (con código que no es `unsafe`). Si se intenta, genera un error de compilación.

También incluye `embassy_sync`, que ofrece primitivas de sincronización con soporte `async` (p. ej. `Channel`, `Signal`)
para la comunicación entre tareas @ref_web_embassy_sync, y `embassy_usb`, para dar soporte USB al código con una API de
nivel bajo @ref_web_embassy_usb.

=== Bibliotecas principales
<sec_herramientas_código>

Fuera del ecosistema de Embassy, se usan varias bibliotecas. A continuación se explican las principales.

`defmt` es una biblioteca de _logging_ que permite enviar mensajes de la placa de desarrollo a la computadora sin
almacenar el texto en la memoria del dispositivo @ref_web_defmt. Transforma los mensajes automáticamente, asignando al
microcontrolador enviar una representación compacta del mensaje (generalmente con un identificador del tipo de mensaje y
sus argumentos), y al ordenador huésped darle formato. El texto de los mensajes se almacena en las secciones de
depuración del binario, que no se envían al microcontrolador. Un ejemplo de su uso se puede ver en el
@cod_ejemplo_defmt.

#figure(
  ```rust
  use defmt::info;

  info!("Midi note received: {}", note);
  // El texto no se envía al microchip. El mensaje únicamente contiene un identificador del mensaje y el valor de `note`
  ```,
  caption: [Ejemplo del uso de `defmt` para registrar eventos en sistemas embebidos, donde el formateo del mensaje se
    realiza en el huésped en lugar del microcontrolador.],
)<cod_ejemplo_defmt>

`fixed` proporciona tipos para operar con números de coma fija en Rust sin un coste de rendimiento @ref_web_fixed.
`bytemuck` a su vez permite hacer conversiones de tipo que no conllevan modificar la representación en bits de los
datos, como convertir un `Q15` a un `i16` o convertir un vector de `Q15` al vector de `u8` que forman sus bytes
@ref_web_bytemuck.


=== Utilidades
<sec_otras_herramientas>

Se usa Nix para proporcionar los programas necesarios para el proyecto de forma reproducible @ref_web_nix_main. Se usa
para formar el entorno de desarrollo y el entorno de usado por GitHub Actions para ejecutar las pruebas automáticas.
Este proporciona todos los programas usadas en el desarrollo, incluyendo Rust, Octave, Typst y todas las utilidades, y
fija sus versiones. Garantiza que el entorno de desarrollo es el mismo que el de ejecución de pruebas, para cumplir el
@rnf_pruebas.

Se usa _just_ como gestor de comandos, una alternativa a usar una `Makefile` más usada por la comunidad de Rust. Este
proporciona una interfaz fácil (`just test`) para todas las secuencias de comandos comunes usadas en el desarrollo. Para
unificar el uso de las distintas herramientas de formato (`typstyle`, `cargo-fmt`) y _linting_ (`clippy`, `cspell`) se
usa `prek`.

Para calcular los coeficientes de los filtros IIR usados en el ecualizador, se usa el paquete `signal` de GNU Octave.

Como herramientas auxiliares, se usan `cargo-nextest` como ejecutor de pruebas en el ordenador, `cargo-binutils` y
`cargo-bloat` para medir el tamaño del binario compilado para poder optimizarlo, `probe-rs` para escribir el código a la
placa de desarrollo y leer sus mensajes, y `lldb` como debugger.
