#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Configuración
<sec_configuración>

=== Durante la compilación
<sec_configuración_compilación>

Sparklet ha de cumplir el @rnf_rendimiento y el @rf_multi_device simultáneamente. Una dificultad es que algunas
características, como el ecualizador o la lectura de MIDI por USB, pueden ser demasiado pesadas para el CPU o la memoria
de dispositivo menos capaces. Desactivar estas funcionalidades en ejecución con un `if` no es suficiente, porque su
código sigue ocupando memoria. Por ejemplo, para dar soporte a MIDI por USB, se incluye un controlador de USB completo
en el código.

Debido a esto, ciertas características de Sparklet son configurable durante la compilación. Para facilitar este proceso,
todas las opciones se pueden controlar usando el archivo `Config.toml`. Un ejemplo de este archivo se puede ver en la
@cod_config_toml. Se permite:

- Activar y desactivar fácilmente las características del sistema, como la inclusión del ecualizador.
- Modificar los parámetros de la aplicación, como la cantidad de voces a usar por el motor de síntesis.
- Establecer la configuración inicial del dispositivo cuando se enciende, como el ataque, sostenimiento, etc.

#figure(
  raw(read("/code/Config.toml"), block: true, lang: "toml"),
  caption: [Fragmento del archivo de configuración `Config.toml`],
  placement: auto,
)<cod_config_toml>

Para permitir activar y desactivar las características, se usan las _feature flags_ de Rust. Las feature flags permiten
incluir o excluir ciertas secciones de código, bibliotecas, etc @ref_web_rust_features. Se puede hacer que dependan de
si cierta feature flag está activa, de si no está activa, o de si cierta combinación está activa, como se puede ver en
el @cod_ejemplo_feature_flags. Usando feature flags, las siguientes características son configurables:

- El chip a usar: qué hardware abstraction layer y qué pines usar.
- La entrada de MIDI: por un pin usando el formato DIN, por USB, o desactivada.
- La inclusión del ecualizador.
- La capacidad de configurar el sintetizador en ejecución (el ataque, la onda, etc.).

#figure(
  ```rust
  #[cfg(feature = "cheat")]
  fn roll_dice() -> u8 { 6 }

  #[cfg(not(feature = "cheat"))]
  fn roll_dice() -> u8 { rand_range(1..=6) }
  ```,
  caption: [Ejemplo básico del uso de feature flags en Rust mediante atributos #[cfg] para seleccionar distintas
    implementaciones de una función en tiempo de compilación. La implementación no seleccionada se incluye en el binario
    final.],
  placement: auto,
)<cod_ejemplo_feature_flags>

El script `run-with-flags.sh` lee los campos relevantes de `Config.toml` y activa las feature flags correspondientes,
permitiendo que se configuren fácilmente. Este toma como argumento el comando a ejecutar con las flags, de manera que
para construir el código se puede ejecutar `./run-with-flags.sh cargo build --release`, y el script ejecutará a su vez
`cargo build --release --no-default-features --features midi-usb audio-usb [...]`.

`Config.rs` también contiene ciertos números constantes como los valores iniciales del ADSR y la cantidad de voces,
llamados parámetros. Se configuran con un archivo `build.rs`, que se ejecuta antes de la compilación. Este lee el
`Config.toml` y genera un archivo `build_config.rs` con un `struct` que contiene todos los parámetros. `build_config.rs`
se incluye en el código en tiempo de compilación con la macro `include!()`.

=== Durante la ejecución

Sparklet se puede configurar en la ejecución con dos botones y tres codificadores rotatorios, como indica el
@rf_runtime_configuration. Para permitir modificar más de tres parámetros con los tres codificadores, se pagina la
configuración: los codificadores modifican los valores de la página actual, mientras que los botones controlan la
página. El módulo responsable de esta gestión es `ConfigManager`, que mantiene el estado de las páginas, los parámetros
y la página seleccionada, y procesa eventos de configuración.

La tarea de configuración es independiente a la de generación de audio. Lee los componentes asociados por muestreo, por
defecto cada $5 "ms"$. Para los botones, mantiene una máquina de estado simple para aplicar _debouncing_. Para los
codificadores rotativos, usa `Qei` de Embassy, que configura los timers del hardware conectando pines a su canal 1 y 2
para acumular la diferencia de fase a un contador, sin necesidad de tiempo del CPU. Cada muestreo, la tarea de
configuración mide la diferencia del contador con la última muestra, y la envía a `ConfigManager`.

Para transmitir la configuración de forma eficiente, según el @rnf_rendimiento, los cambios en la configuración se
transmiten al resto de módulos con menos frecuencia, por defecto cada $100 "ms"$. Se conecta al resto de módulos usando
un `TripleBuffer`, permitiendo que `ConfigManager` nunca se bloquee al escribir y que el la generación nunca se bloquee
al leer.

La taza de muestreo y de actualización de la configuración son parte de los parámetros configurados con el archivo
`build.rs`, como se explica en la @sec_configuración_compilación.

/* Añadir ejemplo de cómo se propaga la configuración */
