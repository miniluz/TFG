#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Operaciones DSP

Para realizar operaciones DSP de forma eficiente, de acuerdo al @rnf_rendimiento, se usa la biblioteca CMSIS-DSP. En
concreto, se usa la biblioteca `cmsis_dsp` de Rust, que proporciona _bindings_ para CMSIS-DSP. Por cada función de la
biblioteca en C se proporciona una función en Rust que la llama, manteniendo las invariantes que Rust espera. Se hizo un
_fork_ de la biblioteca para poder implementar bindings a otras funciones necesarias, como la función
`biquad_cascade_df1_q15`, que usa el ecualizador.

Como se mencionó en la @sec_inst_dsp, Sparklet usa una interfaz llamada `CmsisOperations` con dos implementaciones, una
basada en Rust (que puede ejecutarse en cualquier plataforma compatible, incluyendo `x86_64`) y una basada en
`cmsis_dsp` (que únicamente puede ejecutarse en un chip ARM @ref_web_cmsis_dsp). Con esta interfaz, cada componente
(oscilador, ADSR, ecualizador, etc.) puede depender de `CmsisOperations` y ser implementado con código independiente de
la plataforma. Así pues, casi todo el código puede ser sometido a una batería de pruebas automáticas ejecutable en el
ordenador de desarrollo automáticamente y en los Workflows GitHub Actions, permitiendo cumplir el @rnf_pruebas.

Para validar que ambas implementaciones son idénticas, se creó una batería de pruebas que se ejecuta en ambas. La
implementación en Rust usa el mecanismo estándar de pruebas. La que usa `cmsis_dsp` usa `embedded-test`, una biblioteca
que para ejecutar pruebas en un sistema empotrado @ref_web_rust_embedded_test. Estas pruebas están definidas en el
módulo `cmsis_interface` con una macro, para garantizar que se ejecutan las mismas pruebas en ambas.

La implementación usando una interfaz no tiene un coste de rendimiento el en sintetizador, por lo que esta abtrascción
no obstaculiza el @rnf_rendimiento. Rust permite, además de usar una tabla de métodos virtuales, implementar tipos
genéricos con la _monomorfización_. Dada una función `f<T>(arg: T)`, con el tipo genérico `T`, si se llama con los tipos
concretos `A` y `B`, se generan dos implementaciones de la función: `f(arg: A)` y `f(arg: B)` @ref_web_rust_generics. En
Sparklet, si se define una función `f<CmsisOperations>()`, ya que el ejecutor de Sparklet únicamente la llama con
`CmsisNativeOperations`, el binario compilado únicamente contendrá la función `f()` usando `CmsisNativeOperations`,
generada automáticamente por el compilador.
