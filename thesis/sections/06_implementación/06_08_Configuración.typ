#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Configuración
<sec_configuración>

=== Archivo de configuración

Sparklet es configurable en su totalidad modificando el archivo `Config.toml`. Un ejemplo de este archivo se puede ver
en la @cod_config_toml:

#figure(
  raw(read("/code/Config.toml"), block: true, lang: "toml"),
  caption: [`Config.toml` por defecto],
  placement: auto,
)<cod_config_toml>

Con este archivo se puede:
- Activar y desactivar fácilmente las características del sistema, como si activar el ecualizador.
- Modificar los parámetros de la aplicación, como la cantidad de voces a usar por el motor de síntesis.
- Establecer la configuración inicial del dispositivo cuando se enciende, como el ataque, sostenimiento, etc.

A continuación se explica los mecanismos que usan este archivo para aplicar la configuración que especifica.

=== Durante la compilación
<sec_configuración_compilación>

Algunas características del ecualizador, como el ecualizador o la lectura de MIDI por USB, son demasiado pesadas para
ejecutarse en controladores menos capaces, ya sea por ocupar demasiados ciclos del CPU o demasiada memoria. Para estas
características, no es ideal desactivarlas en ejecución con un `if`, porque su código sigue ocupando memoria. Por lo
tanto, se da la opción de no incluir el código en el programa compilado.

Rust permite usar _feature flags_ para controlar la inclusión o exclusión de secciones de código, bibliotecas, etc. Se
puede hacer que dependan de si cierta feature flag está activa, de si no está activa, o de si cierta combinación está
activa. Usando feature flags, las siguientes características son configurables:

- El chip a usar: qué hardware abstraction layer y qué pines usar.
- La entrada de MIDI: por un pin usando el formato DIN, por USB, o desactivada.
- La inclusión del ecualizador.
- La capacidad de configurar el sintetizador en ejecución (el ataque, la onda, etc.).

/* Añadir ejemplo mínimo de feature flags */

El script `run-with-flags.sh` lee los campos relevantes de `Config.toml` y activa las feature flags correspondientes,
permitiendo que se configuren fácilmente. Este toma como argumento el comando a ejecutar con las flags, de manera que
para construir el código se puede ejecutar `./run-with-flags.sh cargo build --release`, y el script ejecutará a su vez
`cargo build --release --no-default-features --features midi-usb audio-usb [...]`.

`Config.rs` también contiene ciertos números constantes como los valores iniciales del ADSR y la cantidad de voces,
llamados parámetros. Se configuran con un archivo `build.rs`, que se ejecuta antes de la compilación. Este lee el
`Config.toml` y genera un archivo `build_config.rs` con un `struct` que contiene todos los parámetros. `build_config.rs`
se incluye en el código en tiempo de compilación con la macro `include!()`.

=== Durante la ejecución

Sparklet se configura con dos botones y tres codificadores rotatorios. Para permitir modificar más de tres parámetros
con los tres codificadores, se pagina la configuración: los codificadores modifican los valores de la página actual,
mientras que los botones controlan la página. El módulo responsable de esta gestión es `ConfigManager`, que mantiene el
estado de las páginas, los parámetros y la página seleccionada, y procesa eventos de configuración.

La tarea de configuración es independiente a la de generación de audio. Lee los componentes asociados por muestreo, por
defecto cada $5 "ms"$. Para los botones, mantiene una máquina de estado simple para aplicar _debouncing_. Para los
codificadores rotativos, usa `Qei` de Embassy, que configura los timers del hardware conectando pines a su canal 1 y 2
para acumular la diferencia de fase a un contador, sin necesidad de tiempo del CPU. Cada muestreo, la tarea de
configuración mide la diferencia del contador con la última muestra, y la envía a `ConfigManager`.

Cada cierto tiempo, por defecto cada $100 "ms"$, se publica la nueva versión de la configuración si han habido cambios.
La configuración se transmite a la tarea de generación de audio con un `TripleBuffer`, permitiendo que `ConfigManager`
nunca se bloquee al escribir y que el la generación nunca se bloquee al leer.

La taza de muestreo y de actualización de la configuración son parte de los parámetros configurados con el archivo
`build.rs`, como se explica en la @sec_configuración_compilación.

/* Añadir ejemplo mínimo */

/* Añadir ejemplo de cómo se propaga la configuración */
