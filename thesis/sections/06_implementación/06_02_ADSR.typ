#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

#show math.equation.where(block: true): set text(14pt)

== Envolvente ADSR

Un oscilador siendo activado y desactivado con cada nota no genera un sonido agradable. Cuando se toca una nota, empieza
instantáneamente al máximo volumen, y cuando se deja de tocar, para instantáneamente. Estos cambios repentinos se pueden
escuchar como clics, y no tienen un carácter musical.

El envolvente de ataque, decaimiento, sostenimiento y relajación (_attack, decay, sustain, release_ o ADSR) suaviza esta
transición. Es una señal que modula la amplitud de la onda que devuelve el oscilador, como se ve en la
@eq_adsr_modulación.

$
  "salida"[n] = "salida_oscilador"[n] times "salida_adsr"[n]
$
<eq_adsr_modulación>

Se divide en cuatro etapas, como se puede ver en la /* TODO */. Estas son configurables para dar forma al sonido,
generalmente para conseguir imitar un instrumento o dar el carácter buscado a la nota. Son:

/* TODO: insertar imagen de curva ADSR */

+ Ataque: Cuando se toca la nota, pasa de tener amplitud 0 a su máximo volumen. Permite aproximar los sonidos de varios
  instrumentos: una guitarra tiene un ataque corto, mientras que un violín tiene un ataque largo. El ataque, el
  decaimiento y la relajación suelen ser configurables tanto en longitud como en forma, permitiendo que el volumen
  crezca de forma lineal, creciendo más al principio o creciendo más al final.
+ Decaimiento: Velocidad con la que la nota decae al nivel de sostenimiento. Por ejemplo, en instrumentos como la flauta
  o la guitarra es largo, mientras que en una marimba es corto.
+ Sostenimiento: Volumen (como porcentaje del volumen máximo) al que se mantiene la nota indefinidamente mientras sea
  sostenida. Por ejemplo, para aproximar una flauta o un violín sería de casi 100%, ya que pueden mantener la máxima
  intensidad indefinidamente. Sin embargo en instrumentos de cuerda pulsada como el piano o la guitarra suele ser 0%, ya
  que la cuerda lentamente pierde energía hasta que deja de emitir sonido.
+ Relajación o desvanecimiento: Velocidad con la que la nota decae del nivel de sostenimiento a 0 una vez se libera la
  tecla. Su longitud depende más de la técnica que del instrumento: un flautista puede dejar que la nota se desvanezca
  gradualmente, o puede cortarla repentinamente.

=== Derivación matemática.

La matemática usada para el ADSR fue inspirada por #cite(<ref_web_adsr>, form: "prose"). Sin embargo, aunque el proyecto
permite descargar el código, no ofrece una licencia, por lo que tanto la derivación de las fórmulas como la
implementación fueron realizadas de forma independiente. Su idea central es emular cómo los sintetizadores analógicos
realizaban esta transición: emulando el cambio de energía de un condensador.

Esta sección contiene la derivación de las fórmulas que se usan. Sin embargo, es suficiente con saber lo siguiente: la
amplitud $y_n$ de la curva ADSR en la muestra $n$ se calcula aplicando una base $B$ y coeficiente $C$ a la amplitud de
la muestra anterior $y_(n-1)$ como se indica en la @eq_decay_b_c. La base y el coeficiente se calculan en base al valor
inicial $y_0$, el valor objetivo $T_0$, la cantidad de muestras que toma la transformación $n$, y un parámetro llamado
_target ratio_ $r$ que controla qué tan lineal o exponencial es el decaimiento, como se muestra en
@eq_base_coefficient_t_r. Además, pueden ser almacenados en un Q15 ya que su valor absoluto nunca es mayor a 1 en los
casos que necesitamos, como se muestra en @eq_b_c_range. La derivación en detalle se encuentra en un anexo bajo la
@sec_derivación_ADSR.

/* TODO: Insertar gráficos */

$
  y_n = B + y_(n-1) times C
$
<eq_decay_b_c>

$
  C = (inline(r / (1 + r)))^(1/n)
  \
  \
  T = y_0 + (T_0 - y_0) (1 + r)
  \
  \
  B = T times (1 - C)
$<eq_base_coefficient_t_r>

$
  forall r > 0, n > 0 : #h(1em) 0 < C < 1
  \
  \
  forall r > 0, n >= 1, (y_0, T_0) in {(0,1), (1,0)} : -1 <= B <= 1
$<eq_b_c_range>

=== Implementación

El coponente `ADSR` se compone de su máquina de estado, `ADSRState`, su configuración, `ADSRConfig`, y el estado de su
condensador modelado, `Capacitor`.

`Capacitor` sencillamente almacena:
- la carga (amplitud) actual (`current`),
- la carga (amplitud) objetivo (`target`),
- el $B$ y $C$ que usa la etapa de ataque (`rise_base_and_coefficient`),
- el $B$ y $C$ que usan las etapas de decaimiento y relajación (`fall_base_and_coefficient`).
- y el estado: `Charging`, `ReachedTarget`, `Discharging` o `QuickDischarging`.

Los métodos principales que expone son `set_target` y `quick_discharge`. La segunda pone el objetivo a 0 y lo descarga
de forma casi inmediata. `QuickDischarge` se usa cuando se tocan más notas de las que el programa soporta, para liberar
lo más rápido posible una voz sin generar ruido. Usa una $B$ y $C$ específicas.

La configuración `ADSRConfig` almacena:
- el nivel que se mantiene en la etapa de decaimiento (`sustain_level`),
- la amplitud objetivo, determinada por la velocidad de la nota (`velocity_amplitude`) #footnote[La velocidad es lineal
    entre 0 y 127, pero el volumen de un sonido tiene relación logarítmica con su amplitud. En situaciones como esta,
    donde se necesita una amplitud con volumen lineal controlado por un valor lineal, se usa la tabla
    `DB_LINEAR_AMPLITUDE_TABLE`.
  ],

La máquina de estados `ADSRState` tiene los siguientes estados:
- `Idle`, que emite salida cero hasta que se activa con el método `play`, pasando al estado `Attack`,
- `Attack`, que pone el objetivo de `Capacitor` a `velocity_amplitude` hasta que llega a `ReachedTarget`, pasando al
  estado `Decay`,
- `Decay`, que pone a `sustain_level` hasta que llega a `ReachedTarget`, pasando al estado `Decay`,
- `Sustain`, que pone a `sustain_level` indefinidamente. Para pasar a `Release`, se ha de llamar el método
  `stop_playing` (éste pasa de cualquier etapa a `Release`).
- `Release`, que pone a 0 hasta que llega a `ReachedTarget`, pasando al estado idle.
- `QuickRelease`, que activa el `quick_discharge` de `Capacitor` hasta que llega a `ReachedTarget`, pasando al estado
  idle.

/* TODO diagrama máquina de estados */

`ADSRState` expone un método `progress` que progresa el estado una muestra y devuelve su amplitud, aparte de los métodos
`play`, `stop_playing` y `quick_release`.

`ADSR` sencillamente orquesta estos componentes, exponiendo la mayoría de funciones de sus componentes
(`ADSRState::play`, `ADSRConfig::set_attack`, ...) además de otros métodos útiles como `retrigger`, que vuelve a tocar
una nota que ya está siendo tocada, y `get_samples`, que copia las siguientes `LEN` muestras a un `buffer` de entrada.

==== Configurabilidad

Para cumplir con el @rf_adsr, es necesario que sea configurable. Una configuración ADSR puede ser representada como dos
números: la base $B$ y el coeficiente $C$, cada uno almacenable en un Q15. Una tabla de 256 conjuntos de estos dos
números fueron generados para configurar el ataque, usando $r_t = 1,5$ y $t$ siendo de entre 0.01 y 5 segundos (entre
480 y 240.000 muestras). El tiempo de cada configuración $c$ entre 0 y 255, dando un total de $|C| = 256$
configuraciones, con un parámetro $r$ que controla qué tan lineal o exponencial es la subida en tiempo se determina con:

$
  t(c) = t_0 + (t_f - t_0) * (e^(r + (ln(e^r) + 1) - r) * i / |C| - e^r
$
<eq_t_for_c>
/* TODO: Entender cómo funciona esto. */
/* TODO: poner gráficos mostrando cómo se ven los distintos target ratios. */

Se usó la misma técnica para generar la tabla del release, usando $r_t = /* TODO */$ y $t$ siendo de entre /* TODO */.
