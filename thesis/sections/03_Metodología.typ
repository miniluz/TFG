#import "@preview/deal-us-tfc-template:1.0.0": *

= Metodología
<sec_metodología>

== Metodología del desarrollo
<sec_metodología_del_desarrollo>

El proyecto comenzó con un análisis de los competidores para hacer una recogida de las funcionalidades que Sparklet
podría tener. Las funcionalidades elegidas y el resto de requisitos del proyecto fueron divididos en fases. Dichas fases
se ejecutan secuencialmente. Cada una conlleva la investigación necesaria para realizarla, el desarrollo de los
requisitos que recoge, las creación de pruebas automáticas apropiadas, y la ejecución de pruebas manuales de ser
necesario para confirmar el funcionamiento del sintetizador.

Como política de ramas, los commits se hacen directamente sobre `main`. Al ser un proyecto en el que sólo trabaja un
desarrollador, no se considera necesario usar otras ramas. Usando _workflows_ de GitHub Actions se valida cada _push_
que el código compila, que todas las pruebas pasan, que el formato del código es correcto, y que no hay errores
ortográficos en el documento.

En cuanto a la política de _commits_, se realizan commits unitarios, es decir, cada commit corresponde al paso de una
versión funcional del código a otra. El mensaje del commit es una descripción corta de los cambios realizados en él.
Nuevamente, al ser un proyecto con un sólo desarrollador, no se considera necesario usar una política más compleja.

== Estrategia de pruebas
<sec_estrategia_de_pruebas>

Las pruebas se realizan de manera paralela al desarrollo, con las funcionalidades siendo probadas mientras se
implementan. El proyecto usa tres tipos de pruebas:

+ Los generadores de código imprimen a `stderr` comprobaciones de la validez del código que generan, que son revisadas
  manualmente al ejecutarlos. Por ejemplo, para validar la onda de seno, se muestran el nivel de algunas muestras junto
  a sus valores esperados, como que el nivel de la primera muestra es 0. Éstas son revisadas manualmente al ejecutar la
  tabla.

+ La lógica de los módulos del proyecto se somete a pruebas unitarias y ocasionalmente de integración, dependiendo de la
  necesidad según el criterio del desarrollador. Se intenta generar la mínima cantidad de pruebas que garanticen la
  funcionalidad del sistema. Al momento de probar la salida de audio y otros componentes similares, se prueban las
  propiedades del sistema en lugar de la salida en sí misma. Por ejemplo, para validar el oscilador, se estima la
  frecuencia con cruces por cero y se compara con la esperada. Éstas son ejecutadas automáticamente antes de hacer un
  commit, además de en un workflow de GitHub Action cada vez que se hace push.

+ Desde que el código ha sido capaz de leer MIDI y transmitir audio por USB, ha sido ejecutado en la placa de desarrollo
  manualmente para evitar regresiones. Se hace de forma regular durante el desarrollo, pero además se realiza una prueba
  rigurosa de toda su funcionalidad al final de cada fase antes de su cierre.

== Herramientas y tecnologías
<sec_herramientas_y_tecnologías>

=== Código
<sec_herramientas_código>

/* TODO! Hablar de como Rust está en crecimiento para el desarrollo empotrado. */
/* TODO! Hablar de las ventajas de no usar un RTOS */

Para el desarrollo, se usa el ecosistema de Embassy en Rust. Embassy es un _framework_ para el desarrollo empotrado en
Rust usando un ejecutor asíncrono, resultando en programas sin _runtime_, _garbage collector_, ni _RTOS_
@ref_web_embassy. Permite crear programas empotrados con paralelismo usando la sintaxis asíncrona (`async` y `await`) de
Rust para crear tareas, que se convierte en máquinas de estado que comparten el tiempo de ejecución rindiendo el control
al resto de tareas.

Incluye además _hardware abstraction layers_, APIs de Rust que abstraen las capacidades del hardware usado (p. ej.
entrada, salida, _pull-ups_) usando el sistema de tipos de Rust para garantizar que los estados inválidos del hardware
generan fallos durante la compilación. Por ejemplo, en el hardware es imposible activar las interrupciones en los pines
`PA5` y `PB5` simultáneamente, ya que comparten el canal de interrupciones. Por eso la API para crear una entrada con
interrupciones (`ExtiInput`) consume un _struct_ sin datos `EXTI5`, y únicamente se puede obtener uno. Esto hace que
intentar usarlo para dos pines sea un error de compilación, y no conlleva un coste de rendimiento en la ejecución.

El ecosistema también incluye `embassy_sync`, que ofrece primitivas de sincronización con soporte `async` (p. ej.
`Channel`, `Signal`) para la comunicación entre tareas @ref_web_embassy_sync, y `embassy_usb`, para dar soporte USB al
código con una API de nivel bajo @ref_web_embassy_usb.

Adicionalmente, se usa `defmt`, una biblioteca de _logging_ que permite enviar mensajes de la placa de desarrollo a la
computadora sin almacenar el texto en la memoria del dispositivo @ref_web_defmt. Esta biblioteca transforma los mensajes
automáticamente, asignando al microcontrolador enviar una representación compacta del mensaje (generalmente con un
identificador del tipo de mensaje y los argumentos del mensaje), y al ordenador dar formato al mensaje, manteniendo el
texto en las secciones de depuración del binario, que no se envían al microcontrolador.

También se usa la biblioteca `fixed` para operar con números de coma fija en Rust @ref_web_fixed y la librería
`bytemuck` @ref_web_bytemuck para hacer conversiones de tipo entre bytes que no requieren de una operación (p. ej. de
una matriz de `Q15` a su matriz de bytes `u8` correspondiente).


El compilador de Rust permite realizar compilador cruzada @ref_web_rust_cross, permitiendo que el mismo código sea
compilado a `x86_64` (la arquitectura del ordenador usado para el desarrollo) y a `thumbv7em` (_ARM Cortex M7_, la
arquitectura de la placa de desarrollo). Esto se aprovecha moviendo todo el código posible a _crates_ (paquetes) que son
independientes del hardware, permitiendo que la mayoría del código sea probado en plataformas `x86_64` y aún sea
compilable para el microcontrolador. Ésto es fundamental para la experiencia del desarrollo y la metodología de pruebas,
permitiendo desarrollar sin necesidad de la placa y automatizar las pruebas.

=== Utilidades
<sec_otras_herramientas>

Se usa Nix para proveer los programas necesarios para el proyecto de forma reproducible. Se usa para formar el entorno
de desarrollo y el entorno de usado por GitHub Actions @ref_web_nix_main. Éste provee todos los programas usadas en el
desarrollo, incluyendo Rust, Octave, Typst y todas las utilidades, y fija sus versiones.

Se usa _just_ como gestor de comandos, una alternativa a usar una `Makefile` más usada por la comunidad de Rust. Este
provee una interfaz fácil (`just test`) para todas las secuencias de comandos comunes usadas en el desarrollo. Para
unificar el uso de las distintas herramientas de formato (`typstyle`, `cargo-fmt`) y _linting_ (`clippy`, `cspell`) se
usa `prek`.

Para calcular los coeficientes de los filtros IIR usados en el ecualizador, se usa el paquete `signal` de GNU Octave.

Como herramientas auxiliares, se usan `cargo-nextest` como ejecutor de pruebas en el ordenador, `cargo-binutils` y
`cargo-bloat` para medir el tamaño del binario compilado para poder optimizarlo, `probe-rs` para escribir el código a la
placa de desarrollo y leer sus mensajes, y `lldb` como debugger.
