**Preludio**

He dividido el desarrollo en fases a las que he asignado periodos de tiempo para completarlas. Cada fase tiene una serie de tareas a realizar, la investigación que se ha de realizar para cumplirla y de ser posible herramientas útiles para el proceso que ya he determinado.

El TFG será redactado en Latex (ya la he usado antes bastante), y lo redactaré durante el desarrollo de cada fase aparte de tomar notas para no dejarlo todo para el final. Esto resultará en un documento muy partido (se notará que se hizo en partes con diferentes focos), por lo que doy un tiempo para pulirlo.

**Evaluación general del riesgo**

El plan propuesto da bastante tiempo a cada etapa, al estar empezando en septiembre en lugar de en el segundo cuatrimestre.

Voy a incorporar al menos 4 horas de trabajo en el TFG a la semana, dos en cada día en el que tengo menos clases. Además veo probable que algún día trabaje por la tarde o aporte más. La planificación está hecha considerando que cada mes tendrá por lo tanto al menos 16 horas de trabajo.

Tomando esto en cuenta, he intentando sobreestimar con un factor de al menos 2 el tiempo que toma cada fase. Además, he tomado en cuenta las fechas de los exámenes, he reservado un mes y medio específicamente a la redacción del TFG, y he asignado un mes a documentar y mejorar la usabilidad, permitiendo la mayor cantidad de hardware posible, para realmente aportar al estado del arte, que creo que son ámbitos fáciles de subestimar. Con todo esto, planifico que el trabajo estará acabado a finales de abril.

Considero poco probable que me retrase en relación al horario establecido. Pero, en caso de que lo haga, la ampliación, e incluso el filtrado de ser necesario, son opcionales, y pueden ser recortadas sin dejar de aportar al estado del arte. Lo mismo pasa con la configurabilidad: está planeado emplear bastante esfuerzo en pulirla lo más posible, pero con menos trabajo se sigue superando el estado del arte.

Finalmente, siempre queda el mes de mayo y la primera semana de junio para hacer cualquier cambio que sea necesario y trabajar más horas. Aunque en mayo hay exámenes, que ocuparán mi tiempo, considero que es un margen amplio.

Por lo tanto, considero que si hay un retraso con alguna etapa la sobreestimación de la siguiente debería dar un margen para resolverlo. En caso de que no sea así, está el colchón de la ampliación, del mes y medio de tiempo no usado, y en última instancia de la configurabilidad o el filtrado básico.

El mayor riesgo al proyecto es que sea imposible usar Rust, ya que es el lenguaje con el que soy más familiar y el proyecto tiene investigadas muchas librerías de Rust que serían más complicadas de encontrar y trabajar en C. Además que creo que la compilación configurable de manera fácil y accesible a distintos dispositivos se puede dificultar bastante.

Considero esta posibilidad muy remota, ya que Rust tiene una comunidad de desarrollo embedido muy amplia y he encontrado librerías que ya implementan los mayores riesgos que he identificado (no poder usar las instrucciones DSP, el uso de los pines, la lectura de MIDI, etc.)

Pero, por si acaso, la primera fase verifica si Rust puede ser usado. En caso de completarse, garantiza el pode usar las librerías identificadas, o al menos la posibilidad de copiar las que no son críticas (por ejemplo, copiar una de filtrado para que use las instrucciones DSP).

**Fases**

**Fase de prueba de Rust**

(Septiembre)

1. Hacer un proyecto en Rust que compile al chip
2. Emular el chip o algo similar con QEMU para poder desarrollar (y ejecutar pruebas) sin tener conectada la máquina
3. Hacer que se compile usando instrucciones DSP
4. Hacer un proyecto en Rust que lea de un pin y escriba a otro, encendiendo un LED
5. Hacer un proyecto que escriba un stream a un pin y que lea un pin como un stream
6. Hacer un proyecto que ejecute pruebas dummy (sumar números, por ejemplo) en código que se pueda compilar al chip. Idealmente deberían ser ejecutables tanto en QEMU como en el chip real.
7. Leer el libro de Rust embedido

Investigación necesaria:

- Compilación de Rust al chip y herramientas de Rust para ponerles los datos.
- Cómo hacer debugging al chip.
- Cómo compilar a la familia ARM Cortex aprovechando las instrucciones DSP.
- Cómo emular el chip con QEMU.
- Cómo ejecutar pruebas en el chip de QEMU (o en la máquina host con cross-compilation).
- Cómo leer y escribir a pines.

Herramientas útiles:

- El libro de Rust embedido ([https://docs.rust-embedded.org/book/](https://docs.rust-embedded.org/book/ "https://docs.rust-embedded.org/book/")) que parece detallar la mayoría de este proceso
- La librería CMSIS-DSP [https://docs.rs/cmsis_dsp/latest/cmsis_dsp/](https://docs.rs/cmsis_dsp/latest/cmsis_dsp/ "https://docs.rs/cmsis_dsp/latest/cmsis_dsp/"), que provee bindings de Rust para la librería [https://arm-software.github.io/CMSIS_5/DSP/html/index.html](https://arm-software.github.io/CMSIS_5/DSP/html/index.html "https://arm-software.github.io/CMSIS_5/DSP/html/index.html"), que se usa para trabajar con los ARM Cortex
- Los videos de [https://www.youtube.com/@therustybits/videos](https://www.youtube.com/@therustybits/videos "https://www.youtube.com/@therustybits/videos"), un programador de embedido experimentado en C hablando de cómo transicionar de C a Rust. Incluye programación básica y programación asíncrona (aunque no creo que la asíncrona me haga falta, puede ser necesaria para realizar las tareas I/O de manera eficiente)
- [https://embassy.dev/](https://embassy.dev/ "https://embassy.dev/") parece ser una librería súper amplia que puede resolver muchos problemas, y el STM32 tiene soporte de primera clase. Está diseñada para ser usada en real-time, tiene un bootloader y parece proveer USB. [https://lib.rs/crates/daisy-embassy](https://lib.rs/crates/daisy-embassy "https://lib.rs/crates/daisy-embassy") también provee soporte para la Daisy.

Me parece poco probable que no se pueda usar Rust. Rust soporta assembly in-line, incluyendo la arquitectura ARM. Así que en el peor de los casos se pueden escribir abstracciones sobre eso. Pero parece que muchas librerías ya existen para manejarlo.

En caso de que algunos de estos falle, el proyecto se puede realizar en C o en Arduino, pero prefiero C porque es el lenguaje con el que soy más familiar. Esto llevaría a un desarrollo menos ergonómico al dificultar integrar librerías de terceros, y podría alargar el resto de fases. En este caso hacer un sintetizador funcional, o sólo con un filtro de low-pass, pero que tenga la compilación y el ensamblaje bien documentado, sería lo ideal.

Continuar con Rust sería lo mejor porque permite:

1. Compilar a varias targets cambiando pocos archivos (ej. dónde empieza la RAM, dónde empieza el código), por lo que podemos proveer archivos para varios chips STM. En C entiendo que esto es más difícil.
2. Tiene macros en tiempo de compilación muy potentes, por lo que también podríamos poner archivos para configurar cosas como qué pines usar para qué cosas y cuáles son los rangos de voltaje esperados para los potenciómetros (en tiempo de compilación). En C, tendríamos que hacer que la gente toque el código fuente para cambiar constantes para conseguir el mismo resultado.
3. Usar feature flags para desactivar o activar partes de la funcionalidad para permitir compilar a chips menos potentes (ej. los filtros).
4. Es un lenguaje memory-safe, por la que toda la categoría de bugs de memoria (use after free, null pointers, etc.) no son posibles.
5. Es un lenguaje con muchas abstracciones de cero coste, lo que permite el uso de patrones como módulos, interfaces, etc, enumerados, programación funcional, etc. sin tener coste adicional sobre el código equivalente en C.
6. Tiene un gestor de paquetes, lo que facilitaría mucho al usuario descargar las librerías usadas. Además, su librería de paquetes provee mucha de la funcionalidad necesaria para el proyecto (MIDI, USB audio, etc) con código abierto.
7. Es un lenguaje que me interesa y el lenguaje de bajo nivel con el que soy más familiar por diferencia.

**Fase de lectura MIDI**

(Octubre)

1. Hacer un componente que lea MIDI desde los pines.
2. Hacer un componente que de una entrada MIDI saque una serie de notas que están siendo tocadas actualmente (tomando en cuenta una polifonía máxima, ej. si se presionan 5 notas que sólo se "toquen" las últimas 4).
3. Conectar esos dos componentes.

Investigación necesaria:

- Cómo hacer una abstracción de leída "serial" sobre los pines (un stream, no un bit a la vez)
- Cómo generar MIDI en base a esas entradas seriales
- Cómo realizar la lógica de la polifonía de manera eficiente en memoria
- Cómo hacer código eficiente para embedido real-time
- Cómo realizar componentes que sean conectables que sean eficientes en embedido, pero que estén suficientemente aislados para poder realizar pruebas
- Cómo realizar pruebas más complejas

Preguntas a priori:

- ¿Puedo o debo dejar que se trabaje sobre el stack directamente? ¿O sería mejor definir singletons globales para las cosas que se puedan acceder?
- En el caso de definir singletons globales ¿Cómo garantizo que no hayan condiciones de carrera? ¿Es necesario introducir un mutex o arc-mutex?
- ¿Cómo funciona la concurrencia o asincronía en embedido? Entiendo que es por interrupciones hardware, pero ¿cómo manejo eso en mi código? ¿Cómo se conecta la lógica "core" que siempre se ha de ejecutar (síntesis, filtros) de la lógica que se ha de ejecutar para leer dispositivos hardware (como el MIDI)?

Herramientas útiles:

- [https://lib.rs/crates/wmidi](https://lib.rs/crates/wmidi "https://lib.rs/crates/wmidi"), específicamente creado para entornos embedidos
- [https://docs.rust-embedded.org/book/](https://docs.rust-embedded.org/book/ "https://docs.rust-embedded.org/book/")
- Un artículo sobre hacer Rust real-time, [https://codezup.com/how-to-build-a-real-time-embedded-system-with-rust/](https://codezup.com/how-to-build-a-real-time-embedded-system-with-rust/ "https://codezup.com/how-to-build-a-real-time-embedded-system-with-rust/")
- [https://github.com/chris-zen/kiro-synth](https://github.com/chris-zen/kiro-synth "https://github.com/chris-zen/kiro-synth") puede ser una buena referencia, ya que ya es un sistema que funciona en embedido
- [https://embassy.dev/](https://embassy.dev/ "https://embassy.dev/")

No considero que esta etapa tenga un riesgo particular.

**Fase del sintetizador**

(Noviembre)

1. Hacer un componente de envelope ADSR (attack decay sustain release).
2. Hacer un componente sintetizador configurable, que reciba frecuencias y genere el sonido tomando en cuenta la envelope. También deberías poder elegir entre diferentes tipos de onda (cuadrada, triangular, seno, sierra).
3. Conectar las notas siendo tocadas al sintetizador para que genere el sonido.

Investigación necesaria:

- Cómo realizar síntesis de varios osciladores de manera eficiente y estable (estable significando que no haya popping al cambiar de nota de manera repentina en la medida de lo posible)
- Cómo puedo obtener el tiempo que ha pasado para operar la onda y el envelope ADSR
- Cómo se crean las envelopes ADSR
- Cómo se pueden usar las instrucciones DSP para hacer esto más eficiente
- Cómo configurar la sample rate (y quizás el buffer size), y cuáles son adecuados.

Herramientas útiles:

- [https://github.com/chris-zen/kiro-synth](https://github.com/chris-zen/kiro-synth "https://github.com/chris-zen/kiro-synth")
- [https://docs.rust-embedded.org/book/](https://docs.rust-embedded.org/book/ "https://docs.rust-embedded.org/book/")
- [https://embassy.dev/](https://embassy.dev/ "https://embassy.dev/")

No considero que esta etapa tenga un riesgo parcticular, ya que tengo proyectos de referencia. Incluso podría delegar la síntesis a una librería de ser necesario, pero es una parte que preferiría escribir yo.

**Fase de configuración**

(Diciembre y enero) (doy dos meses por los exámenes)

1. Hacer un componente de lectura de potenciómetros/interruptores que tome en cuenta el ruido/rebote y minimice la cantidad de actualizaciones, para poder hacer caché de lo generado para la configuración actual.
2. Conectar el componente de lectura a la configuración del sintetizador y el envelope ADSR. Seguramente involucre cambiar su código.

Investigación necesaria:

- Cómo leer entradas en embedded y procesarlas para evitar ruido
- Cómo hacer caché de las opciones de configuración de manera eficiente.
- Cómo hacer que los pines de las entradas de la configuración sean configurables

Herramientas útiles:

- [https://docs.rust-embedded.org/book/](https://docs.rust-embedded.org/book/ "https://docs.rust-embedded.org/book/")
- [https://embassy.dev/](https://embassy.dev/ "https://embassy.dev/")

No considero que la etapa tenga un riesgo en particular, y espero poder construirla fácilmente con la información que tengo de la fase de MIDI y síntesis de conectar componentes que se ejecutan siempre a componentes I/O.

**Fase de output**

(Diciembre y enero, immediatamente después a la de configuración)

1. Hacer un componente que dada una señal de audio de un output, quizá por audio an analógico o quizá con USB (o ambas).

Investigación necesaria:

- Cómo hacer una salida analógica (Mi padre me comentó que hay un circuito simple que pasa de varios pines digitales a análogo con resistencias del mismo tamaño). Si la board tiene un pin de salida analógica, me ahorro esto.
- Cómo hacer salida por USB (quizá puedo usar el puerto que trae el mismo dispositivo, el que se usa para serial, o quizá lo suyo es usar pines).
- Cómo hacer que los pines de salida sean configurables.

Herramientas útiles:

- [https://docs.rust-embedded.org/book/](https://docs.rust-embedded.org/book/ "https://docs.rust-embedded.org/book/")
- [https://embassy.dev/](https://embassy.dev/ "https://embassy.dev/")
- [https://lib.rs/crates/embassy-usb](https://lib.rs/crates/embassy-usb "https://lib.rs/crates/embassy-usb") o [https://lib.rs/crates/usbd-audio](https://lib.rs/crates/usbd-audio "https://lib.rs/crates/usbd-audio")

No considero que la etapa tenga un riesgo en particular. USB seguramente sea el más complejo pero hacer una salida analógica no debería ser muy difícil.

**Fase de filtrado**

(Primera mitad de febrero)

1. Hacer un componente de filtrado low-pass configurable. Sería configurable la frecuencia, la pendiente y el Q. Quizá es suficiente con calcular en tiempo real el filtro, pero quizá haga falta hacer un caché de él dados los parámetros de entrada.
2. Conectar la salida del sintetizador a los sistemas de filtrado antes de que pase al output.
3. Añadir una opción para que la frecuencia de corte siga la envelope ADSR.

Investigación necesaria:

- Cómo hacer filtros musicales (que tengan rampas lineales en decibelios por octava) y configurables. Creo que son filtros IIR "biquad", porque la lectura estableció que los filtros FIR no tienen phase shift y siempre tienen retraso. Claude me habló de "Butterworth", "Chebyshev", "Bessel" y "Linkwitz-Riley."
- Cómo se pueden usar las intrucciones DSP para hacer esos filtros más eficientes.
- Cómo hacer que los filtros sean eliminables con feature flags para permitir ejecutar el código en hardware menos potente

Herramientas útiles:

- [https://lib.rs/crates/biquad](https://lib.rs/crates/biquad "https://lib.rs/crates/biquad") (pero habría que garantizar que usa las instrucciones DSP)
- [https://lib.rs/crates/signalsmith-dsp](https://lib.rs/crates/signalsmith-dsp "https://lib.rs/crates/signalsmith-dsp")
- [https://docs.rust-embedded.org/book/](https://docs.rust-embedded.org/book/ "https://docs.rust-embedded.org/book/")
- [https://embassy.dev/](https://embassy.dev/ "https://embassy.dev/")
    
    **Redacción del TFG y mejoras al código**
    
    (Segunda mitad de febrero y marzo)

Este mes lo dedico más que nada a pulir el TFG. Debería involucrar tomar toda la información ya escrita y hacerla más accesible y conexa, y ponerla en un orden que tenga sentido. Además debe involucrar asegurar una cantidad razonable de citas, y que la información esté bien repartida entre el desarrollo, conocimiento previo y demás.

Considero inevitable que durante el progreso del mes encontraré información que contradiga lo que he hecho, o que tenga código algo malo o que no se corresponda con las buenas prácticas, y esto también corresponde a este mes.

**Fase de documentación y usabilidad**

(Abril)

1. Asegurar que el código es legible y que está bien documentado.
2. Asegurar que la configuración del chip, en la medida de lo posible, se hace fuera del código o en archivos que sólo contengan esos datos.
3. Intentar que el código sea compatible con la mayor cantidad de hardware posible fuera del chip, es decir con diferentes potenciómetros, etc.
4. Documentar todo el proceso y hacer guías de cómo obtener la información necesaria (por ejemplo, cómo conseguir cuándo comienzan los segmentos de memoria, cómo configurar los pines y cómo activar o desactivar componentes).
5. Identificar requisitos mínimos de RAM y espacio de código (para distintas feature flags).
6. Opcionalmente, intentar que el código también se pueda compilar a otros chips ARM. ¡Incluyendo el daisy seed, en caso de que sea posible!

Investigación necesaria:

- Cómo acceder a archivos en tiempo de compilación y usarlos para determinar comportamiento
- Cómo usar feature flags para hacer que parte del código sea opcional
- Cómo usar feature flags para activar feature flags de otras librerías (por ejemplo, la librería de instrucciones DSP requiere de una feature flag si tienes un Arm Cortex M7)
- Cómo compilar a varias arquitecturas con el mismo código de manera ergonómica
- Cómo identificar el RAM y el espacio de código que usa el programa compilado.

Herramientas útiles:

- [https://doc.rust-lang.org/cargo/reference/features.html](https://doc.rust-lang.org/cargo/reference/features.html "https://doc.rust-lang.org/cargo/reference/features.html")
- [https://doc.rust-lang.org/reference/conditional-compilation.html](https://doc.rust-lang.org/reference/conditional-compilation.html "https://doc.rust-lang.org/reference/conditional-compilation.html")
- [https://crates.io/crates/confy](https://crates.io/crates/confy "https://crates.io/crates/confy")

Considero que el riesgo principal de esta etapa es que no se pueda configurar varios chips (ej. que sólo se pueda compilar al M7). De ser así, aunque no sería ideal, se seguría superando el estado del arte, por lo que no sería crítico.

**Fase de ampliación (opcional)**

(Abril)

1. Añadir opciones para que el filtro pueda ser high-pass.
2. Añadir más efectos (reverb, delay distorción, quizá phaser o chorus) que sean viables con el hardware.

Investigación necesaria:

- Cómo hacer los efectos nombrados

Herramientas útiles:

- [https://lib.rs/crates/signalsmith-dsp](https://lib.rs/crates/signalsmith-dsp "https://lib.rs/crates/signalsmith-dsp")
- [https://lib.rs/crates/lanceverb](https://lib.rs/crates/lanceverb "https://lib.rs/crates/lanceverb")

Ya que es una etapa de ampliación, considero que tiene un riesgo muy bajo. De no ser posible o tomar demasiado, se puede ignorar.