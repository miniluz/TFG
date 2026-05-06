#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== CMSIS

Para lidiar con CMSIS, se usa la librería `cmsis_dsp` de Rust, que provee _bindings_ para la librería CMSIS; es decir,
por cada función de la librería en C se provee una función en Rust que hace la misma operación. Se hizo un _fork_ de la
librería para poder implementar bindings a otras funciones necesarias, como la función `biquad_cascade_df1_q15` que usa
el banco de filtros.

Como se mencionó en la @sec_inst_dsp, Sparklet usa una interfaz llamada `CmsisOperations` con dos implementaciones, una
basada en Rust (que puede ejecutarse en cualquier plataforma compatible, incluyendo `x86_64`) y una basada en
`cmsis_dsp` (que únicamente puede ejecutarse en un chip ARM).

=== Pruebas

Para validar que ambas implementaciones son iguales, se ejecuta la misma batería de pruebas en ambas. En `CmsisRust`,
las pruebas se ejecutan usando el mecanismo estándar de Rust. En `CmsisNative`, se implementan con `embedded-test`, una
librería que permite usar el mecanismo estándar de pruebas de Rust en un sistema empotrado con Embassy. Estas pruebas
están definidas en el módulo `cmsis_interface` con una macro, para garantizar que las implementaciones son idénticas.

Sabiendo que las implementaciones son idénticas, cada componente (oscilador, ADSR, filtros...) puede depender de
`CmsisOperations` y ser implementado con código independiente de la plataforma. Así pues, casi todo el código puede ser
sometido a una batería de pruebas automáticas ejecutable en el ordenador de desarrollo automáticamente, e incluso por
GitHub Actions. El único código que ha de ser probado en un sistema empotrado es `CmsisNative`.

Los tipos genéricos en Rust no tienen un coste de rendimiento al ejecutar un programa. Esto es debido a que Rust
_monomorfiza_ los tipos genéricos: dada una función `f<T>(arg: T)`, con el tipo genérico `T`, si se llama con los tipos
concretos `A` y `B` se generan dos implementaciones de la función: `f(arg: A)` y `f(arg: B)`. En este caso, si se define
una función `f<CmsisOperations>()`, ya que el ejecutable de Sparklet únicamente las llama con `CmsisNativeOperations`,
únicamente se genera la función `f()` equivalente que usa `CmsisNativeOperations`, sin coste en ejecución en comparación
a llamar las funciones directamente.
