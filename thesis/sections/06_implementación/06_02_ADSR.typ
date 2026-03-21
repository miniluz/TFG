#import "@preview/deal-us-tfc-template:1.0.0": *

#show math.equation.where(block: true): set text(14pt)

== Envolvente ADSR

Un oscilador siendo activado y desactivado con cada nota no genera un sonido agradable. Cuando se toca una nota, empieza
instantáneamente al máximo volumen, y cuando se deja de tocar, para instantáneamente. Estos cambios repentinos se pueden
escuchar como clics, y no tienen un carácter musical.

El envolvente de ataque, decaimiento, sostenimiento y relajación (_attack, decay, sustain, release_ o ADSR) suaviza esta
transición. Es una señal que modula la amplitud de la onda que devuelve el oscilador. Se divide en cuatro etapas, como
se puede ver en la /* TODO */. Estas son configurables para dar forma al sonido, generalmente para conseguir imitar un
instrumento o dar el carácter buscado a la nota. Son:

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

== Derivación matemática.

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
