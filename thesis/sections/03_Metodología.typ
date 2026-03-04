#import "@preview/deal-us-tfc-template:1.0.0": *

= Metodología
<sec_metodología>

== Metodología del desarrollo
<sec_metodología_del_desarrollo>

El proyecto comenzó con un análisis de los competidores para hacer una recogida de las funcionalidades que Sparklet
podría tener. Las funcionalidades elegidas y el resto de requisitos del proyecto fueron divididos en fases, que se
realizan secuencialmente. Cada fase conlleva la investigación necesaria para realizarla, el desarrollo de los requisitos
que recoge, las creación de pruebas automáticas apropiadas, y la ejecución de pruebas manuales de ser necesario para
confirmar el funcionamiento de Sparklet.

Como política de ramas, los commits se hacen directamente sobre main. Al ser un proyecto en el que sólo trabaja un
desarrollador, no se considera necesario usar una política más compleja. GitHub Actions en cada _push_ valida el formato
del código y ejecuta las pruebas.

En cuanto a la política de _commits_, se realizan _commits_ unitarios, es decir, cada _commit_ deben pasan de una
versión funcional del código a otra versión funcional con los cambios que describe implementados. El mensaje del
_commit_ es una descripción corta de los cambios realizados en él. Nuevamente, al ser un proyecto con un sólo
desarrollador, no se considera necesario usar una política más compleja.

== Estrategia de pruebas
<sec_estrategia_de_pruebas>

Las pruebas se realizan de manera paralela al desarrollo, con las funcionalidades siendo probadas mientras se
implementan. El proyecto usa tres tipos de pruebas:

+ Los generadores de código imprimen a `stderr` comprobaciones de la validez del código que generan, que son revisadas
  manualmente al ejecutarlos.

+ La lógica de los módulos del proyecto es probada con pruebas unitarias y ocasionalmente de integración, dependiendo
  del criterio de desarrollador de la necesidad. Se intenta generar la mínima cantidad de pruebas que garanticen la
  funcionalidad del sistema. Al momento de probar la salida de audio y otros componentes similares, se prueban las
  propiedades de la salida (por ejemplo, para validar el sintetizador por tabla de ondas, se estima la frecuencia con
  cruces por cero y se compara con la esperada)

+ Desde que el código ha sido capaz de leer MIDI y transmitir audio por USB, ha sido ejecutado en la placa de desarrollo
  y probado regularmente durante el desarrollo de cada fase y probado rigurosamente al final de estas manualmente.

Las comprobaciones de validez son revisadas manualmente al ejecutar los generadores. Las pruebas unitarias se ejecutan
regularmente durante el desarrollo y se revisan antes de hacer un commit. Además, se ejecutan en GitHub Actions para
evitar error humano.

== Herramientas y tecnologías
<sec_herramientas_y_tecnologías>

=== Código
<sec_herramientas_código>

TODO! Hablar de como Rust está en crecimiento para el desarrollo empotrado.

Para el desarrollo, se usa el ecosistema de Embassy en Rust. Embassy es un framework para el desarrollo empotrado en
Rust usando un ejecutor asíncrono, resultando en programas sin _runtime_, _garbage collector_, ni _RTOS_
@ref_web_embassy. Permite crear programas empotrados con paralelismo usando la sintaxis asíncrona (`async` y `await`) de
Rust para crear tareas, que se convierte en máquinas de estado que comparten el tiempo de ejecución rindiendo el control
al resto de tareas.

Incluye además _hardware abstraction layers_, APIs de Rust que abstraen las capacidades del hardware usado (p. ej.
entrada, salida, _pull-ups_) usando el sistema de tipos de Rust para garantizar que los estados inválidos del hardware
generan fallos durante la compilación. Por ejemplo, en el hardware es imposible activar las interrupciones en los pines
`PA5` y `PB5` simultáneamente ya que comparten el canal de interrupciones. Por eso la API para crear una entrada con
interrupciones (`ExtiInput`) consume un _struct_ `EXTI5` del que únicamente se puede obtener uno. Esto hace que intentar
usarlo para dos pines sea un error de compilación.

El ecosistema también incluye `embassy_sync`, que ofrece primitivas de sincronización con soporte `async` (p. ej.
`Channel`, `Signal`) para la comunicación entre tareas @ref_web_embassy_sync, y `embassy_usb`, para dar soporte USB al
código con una API de nivel bajo @ref_web_embassy_usb.

Adicionalmente, se usa `defmt`, una librería de _logging_ que permite enviar mensajes de texto de la placa de desarrollo
a la computadora sin almacenar el texto en la memoria del dispositivo ,@ref_web_defmt. Esta convierte los mensajes
automáticamente a una versión mínima (generalmente con un número identificando el tipo de mensaje y los datos
relevantes) y difiere el formateo del mensaje a texto legible a la computadora.

También se usa la librería `fixed` para operar con números de coma fija en Rust @ref_web_fixed y la librería `bytemuck`
@ref_web_bytemuck para hacer conversiones de tipos con coste zero (p. ej. de `[Q15]` a `[i16]`). Para generar los
coeficientes de los filtros IIR, se usa _GNU Octave_ con el paquete `signal` para generar

El compilador de Rust admite cross-compilation @ref_web_rust_cross, permitiendo que el mismo código sea compilado a
`x86_64` (la arquitectura del ordenador usado para el desarrollo) y a `thumbv7em` (_ARM Cortex M7_, la arquitectura de
la placa de desarrollo). Esto se aprovecha moviendo todo el código posible a _crates_ que son independientes del
hardware, permitiendo que código compilado a la placa de desarrollo pueda ser probado automáticas en plataformas
`x86_64`, permitiendo probar mientras se desarrolla sin necesidad de usar la placa y probar en GitHub Actions.

=== Utilidades
<sec_otras_herramientas>

Se usa Nix para garantizar la reproducibilidad de la construcción y proveer los paquetes necesarios para el proyecto,
así como el entorno de desarrollo y el entorno de ejecución de GitHub Actions @ref_web_nix_main. Éste provee todos los
programas usadas en el desarrollo, incluyendo Rust, Octave, Typst y todas las utilidades, y fija sus versiones.

Se usa _just_ como gestor de comandos, una alternativa más moderna y agnóstica a lenguaje que usar `Makefile`. Este
provee alias (`just test`) para procesos más complejos de comandos. Unifica todas las herramientas que se usan en un
sólo comando. Para unificar el uso de los distintos `formatters` (`typstyle`, `cargo-fmt`) y _linters_ (`clippy`,
`cspell`) se usa `prek`.

Como herramientas auxiliares, se usan `cargo-nextest` como ejecutor de pruebas `x86_64`, `cargo-binutils` y
`cargo-bloat` para medir el tamaño del binario compilado para poder optimizarlo, `probe-rs` para escribir el código a la
placa de desarrollo y leer sus mensajes, y `lldb` como debugger.
