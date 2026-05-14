#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

#show math.equation.where(block: true): set text(14pt)

== Envolvente ADSR

Cuando un oscilador se activa y desactiva repentinamente, no genera un sonido agradable. Cuando se toca una nota,
empieza repentinamente al máximo volumen, y cuando se deja de tocar, para instantáneamente. Estos cambios bruscos se
escuchan como clics, que no son compatibles con el @rnf_audio_quality.

El envolvente de ataque, decaimiento, sostenimiento y relajación (_attack, decay, sustain, release_ o ADSR) suaviza esta
transición. Se origina en los sintetizadores analógicos, y se ha convertido en el estándar para controlar la envolvente
de amplitud de un sintetizador @ref_book_music_tutorial. Un envolvente de amplitud es una señal que controla la amplitud
de una onda, y por lo tanto su volumen. En este caso, se usa el ADSR como envolvente de amplitud para la onda que genera
el oscilador, como se ve en la @eq_adsr_modulación.

$
  "salida"[n] = "salida_oscilador"[n] times "salida_adsr"[n]
$
<eq_adsr_modulación>

Se divide en cuatro etapas, como se puede ver en la @fig_adsr. Estas son configurables para dar forma al sonido,
generalmente para conseguir imitar un instrumento o dar el carácter buscado a la nota de forma intuitiva (p. ej. "hacer
el ataque más agresivo") @ref_book_music_tutorial. Son:

#figure(
  include "/figures/adsr.typ",
  caption: [Ilustración de cómo los parámetros de ataque, decaimiento, sostenimiento y relajación determinan la forma de
    un envolvente ADSR.],
  placement: auto,
)<fig_adsr>

+ Ataque: Tiempo que tarda la nota en pasar de estar silenciada a tener su máximo volumen. Ajustar su longitud permite
  aproximar los sonidos de varios instrumentos: una guitarra tiene un ataque corto, mientras que un violín tiene un
  ataque largo.
/* El ataque, el
decaimiento y la relajación suelen ser configurables tanto en longitud como en forma, permitiendo que el volumen
crezca de forma uniforme, que crezca más al principio o que crezca más al final. */
+ Decaimiento: Tiempo que tarda la nota en bajar del volumen máximo al nivel de sostenimiento. Por ejemplo, en
  instrumentos como la flauta es largo, mientras que en una marimba es corto.
+ Sostenimiento: Volumen al que se mantiene la nota indefinidamente mientras sea tocada. Por ejemplo, para aproximar una
  flauta o un violín sería de casi el 100%, ya que esos instrumentos pueden mantener una nota indefinidamente. Sin
  embargo, para imitar una marimba generalmente se usaría un sostenimiento del 0%, ya que la vibración de la barra decae
  naturalmente hasta extinguirse.
+ Relajación o desvanecimiento: Tiempo que tarda el volumen de la nota en bajar del nivel de sostenimiento a cero una
  vez se libera la tecla. Su longitud depende más de la técnica que del instrumento: un flautista puede dejar que la
  nota se desvanezca gradualmente, o puede cortarla repentinamente. En un sintetizador, se suele ajustar según la
  atmósfera que se busca en la composición.

=== Modelo matemático
<sec_adsr_modelo>

Una transición lineal en el volumen se corresponde con un crecimiento exponencial en la amplitud, como se puede ver en
la @fig_vol_lineal, debido a que la percepción del volumen es logarítmica. El envolvente que genera resulta poco útil en
la mayoría de casos, pues se percibe que la energía se concentra al final de la transición al subir el volumen y al
principio al bajarlo.

Una transición lineal en la amplitud genera un envolvente más útil para el ataque, ya que el volumen sube de forma más
rápida al principio, como se ve en la @fig_amp_lineal. Sin embargo, tiene el problema opuesto para el decaimiento: al
principio se pierde poca energía y al final se pierde con demasiada velocidad.

#figure(
  image("/figures/adsr_vol_lineal.png", width: 69%),
  caption: [Amplitud equivalente para una transición lineal del volumen.],
  placement: auto,
)<fig_vol_lineal>
#figure(
  image("/figures/adsr_amp_lineal.png", width: 69%),
  caption: [Volumen equivalente para una transición lineal de la amplitud.],
  placement: auto,
)<fig_amp_lineal>
#figure(
  image("/figures/adsr_condensador.png", width: 69%),
  caption: [La transición conseguida por el modelo matemático #linebreak() del condensador, con $r_t = 0,1$.],
  placement: auto,
)<fig_condensador>

Como solución, se modela matemáticamente el mecanismo que usan los sintetizadores analógicos para controlar el volumen
de una nota: un condensador. La implementación fue inspirada por la de #cite(<ref_web_adsr>, form: "prose").
#footnote[Aunque el proyecto permite descargar el código, no ofrece una licencia, por lo que tanto la derivación de las
  fórmulas como la implementación fueron realizadas de forma independiente.]
El cambio de voltaje de un condensador sigue el decaimiento exponencial. Esta implementación proporciona además un
parámetro adicional _target ratio_ $r$ que permite interpolar entre un decaimiento exponencial y una transición lineal,
para permitir dar la forma deseada al envolvente. Se puede ver en la @fig_condensador la respuesta obtenida, que es
agresiva en el ataque y pierde energía más rápidamente en el decaimiento.

Un condensador sigue el decaimiento exponencial. Para poder modelar este comportamiento con números de coma fija, que no
pueden hacer cálculos exponenciales, se transforman las ecuaciones del decaimiento a una forma recursiva. La forma
recursiva también es eficiente, como pide el @rnf_rendimiento. La derivación en detalle se encuentra en un anexo bajo la
@sec_derivación_ADSR. El resumen es el siguiente: la amplitud $y_n$ de la curva ADSR en la muestra $n$ se calcula en
base a la muestra anterior $y_(n-1)$ de forma recursiva. Este cálculo consiste en multiplicarla por un coeficiente $C$ y
sumarle una base $B$, como se indica en la @eq_decay_b_c. La base y el coeficiente se calculan en base al valor inicial
$y_0$, el valor objetivo $T_0$, y la cantidad de muestras que toma la transformación $n$. A partir del target ratio $r$
ya mencionado se calcula un objetivo corregido $T$, como se muestra en @eq_base_coefficient_t_r. Además, pueden ser
almacenados en un Q15, ya que su valor absoluto nunca es mayor a 1 en los casos relevantes, como se muestra en
@eq_b_c_range.

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

El componente `ADSR` coordina sus subcomponentes: la máquina de estado `ADSRState`, la configuración `ADSRConfig` y el
condensador modelado `Capacitor`. Reexporta la mayoría de funciones de sus componentes (`ADSRState::play`,
`ADSRConfig::set_attack`, ...) y ofrece otros métodos útiles como `retrigger`, que vuelve a tocar una nota que ya está
siendo tocada, y `get_samples`, que copia las siguientes `LEN` muestras a un `buffer` de entrada.

El modelo del condensador `Capacitor` es una máquina de estados pequeña. Tiene los estados `Charging`, `Discharging`,
`ReachedTarget` y `QuickDischarging`. `QuickDischarging` se usa cuando se tocan más notas a la vez de las que el
programa soporta y descarga el condensador de forma casi inmediata (véase la @sec_banco_de_voces). Mantiene la carga
actual (`current`) y objetivo (`target`), aparte de los $B$ y $C$ que usa para cargar (en el ataque) y descargar (en el
decaimiento y relajación). Las $B$ y $C$ de `QuickDischarging` son constantes externas a `Capacitor`. El método
principal de configuración es `set_target`: el condensador internamente calcula si se carga o descarga.

La máquina de estados `ADSRState` es el núcleo de la implementación. Cada estado define su comportamiento con el nivel
de amplitud y su transición a otros estados. Comparten entre sí un `Capacitor` que se usa para suavizar cambios entre
las fases y señales de control, como la velocidad o la configuración del sostenimiento. Este diseño evita
discontinuidades bruscas en la amplitud cuando se cambian parámetros, reduciendo la aparición de artefactos audibles
como clics. Aparte de los estados estándar, `Attack`, `Decay`, `Sustain` y `Release`, tiene un estado `Idle` que
devuelve siempre cero y un estado `QuickRelease` que activa el modo `QuickDischarge` de `Capacitor`, también usado para
la gestión de voces. Cada estado ajusta el objetivo del condensador y avanza su simulación una muestra, como se puede
ver en el pseudocódigo de su implementación en el @cod_adsr_state.

`ADSRState` expone un método `progress` que devuelve la amplitud de la siguiente muestra, cambiando su estado si es
apropiado. Además ofrece los métodos `play`, `stop_playing` y `quick_release`, que son llamados cuando las notas se
tocan o dejan de tocar.

#figure(
  ```rust
  match *self {
      Self::Idle => I1F31::ZERO,
      Self::Attack => {
          capacitor.set_target(velocity);
          let status = capacitor.step();
          if ReachedTarget { *self = Decay; }
          return capacitor.get_level();
      }
      Self::Decay => {
          capacitor.set_target(sustain * velocity);
          let status = capacitor.step();
          if ReachedTarget { *self = Sustain; }
          return capacitor.get_level();
      }
      Self::Sustain => {
          capacitor.set_target(sustain * velocity);
          capacitor.step();
          capacitor.get_level()
      }
      Self::Release => {
          capacitor.set_target(0);
          let status = capacitor.step();
          if ReachedTarget { *self = Idle; }
          capacitor.get_level()
      }
      Self::QuickRelease => {
          capacitor.quick_discharge();
          let status = capacitor.step();
          if ReachedTarget { *self = Idle; }
          capacitor.get_level()
      }
  }
  ```,
  caption: [Lógica de la máquina de estados `ADSRState`],
  placement: auto,
)<cod_adsr_state>


==== Configurabilidad

Para cumplir con el @rf_adsr, es necesario que sea configurable. La configuración se reparte entre `Capacitor` y
`ADSRConfig`. `Capacitor` mantiene la configuración que le es relevante: los $B$ y $C$ usados para cargar (en el ataque)
y descargar (en el decaimiento y la relajación).
#footnote[Una peculiaridad de la implementación es que la longitud del decaimiento y de la relajación no son
  configurables por separado, ya que ambos son modelados como una descarga del condensador. Es intencional, pues es el
  comportamiento de un sintetizador analógico real. Además, encaja con el modelo de configuración por hardware del
  sintetizador, que tiene tres codificadores rotativos: uno para el ataque, uno para el decaimiento y la relajación, y
  uno para el sostenimiento.]
`ADSRConfig` mantiene la información externa al condensador, como el nivel de sostenimiento y la amplitud objetivo,
determinado con la velocidad de la nota.
#footnote[Tanto en el sostenimiento como la velocidad, la amplitud tiene relación lineal con la señal de configuración,
  en lugar del volumen. Experimentalmente, produce un resultado más natural, por motivos similares a los explicados en
  la @sec_adsr_modelo. Usar una relación lineal con el volumen resulta en que las notas tengan un volumen más bajo del
  esperado al tocar más suavemente o usar un sostenimiento bajo. Lo ideal sería usar una curva intermedia, como la del
  modelo del condensador, pero determinar la amplitud de forma lineal es eficiente y suficiente.]

La configuración de la longitud del ataque, el decaimiento y la relajación se representa con un número entre 0 y 255. La
base $B$ y el coeficiente $C$ para todas estas configuraciones se almacenan en una tabla calculada de antemano, ya que
sus cálculos requieren de operaciones de coma flotante que pueden no estar disponibles en el microcontrolador usado. La
generación del ataque usa $r_t = 0,1$, con el tiempo $t$ abarcando entre 0.01 y 5 segundos. La del decaimiento y la
relajación, usa $r_t = 0,2$ con el tiempo $t$ abarcando entre 0.01 y 5 segundos. No se usan pasos homogéneos en el
tiempo, porque el salto entre $50 "ms"$ y $100 "ms"$ de ataque se notaría mucho más que el salto entre $4,5 "s"$ y
$4,55 "s"$. En su lugar, se usa una curva entre lineal y exponencial para facilitar su configuración, dando más
precisión al controlar el tiempo cuando es bajo (p. ej. la configuración que sigue a $10 "ms"$ es $11 "ms"$ y la que
sigue a $1 "s"$ es $1,1 "s"$). El modo `QuickReleasee` usa $r_t = 2$ y $t = 0,01$.
