#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Osciladores
<sec_osciladires>

// TODO: Ilustración de los osciladores.

Los osciladores son donde comienza la síntesis. Estos reciben una frecuencia y generan una señal de audio a dicha
frecuencia. Hay diversos tipos de osciladores, pero uno de los más usados en entornos empotrados es el de tabla de onda
(_wavetable_), debido a su eficiencia. Esta es la razón por la que Sparklet lo usa. La onda deseada se almacena en una
tabla de longitud arbitraria $L$ como se ve en la /* TODO */, y se reproduce a la frecuencia deseada
@ref_book_music_tutorial @ref_book_theory_music.

// TODO: Ilustración de una wavetable.

Dada una tabla de longitud $L = 1000$ y un sistema con frecuencia de muestreo de $f_s = 48.000 "Hz"$. Si cada muestreo
$s$ se usa como íncide de la tabla $s mod 1.000$ para obtener la muestra, se genera una onda con la forma de la tabla a
$f = 48.000 "Hz" div 1.000 = 48 "Hz"$. Si en lugar se avanza por dos cada muestra, usando $2 times s mod 1.000$ como
índice, se envía una onda con frecuencia $f = 48.000 "Hz" div 500 = 96 "Hz"$. En general, la relación entre la
frecuencia $f$ y el incremento del índice $i$ se da con la @eq_incremento @ref_book_music_tutorial
@ref_book_theory_music.

$
  i = (L times f) / f_s
$
<eq_incremento>

Este incremento no es entero para la mayoría de frecuencias. La resolución de frecuencias representables depende de la
cantidad de bits que se use para representar y acumular el incremento. Sparklet usa una tabla de longitud $L = 256$.
Supongamos que se usa un número de 16 bits, en formato UQ8.8 (es decir, un número de coma fija sin signo donde 8 bits se
dedican a la parte entera y 8 dígitos a la parte fraccionaria). Esto da una resolución de
$48.000 "Hz" div 2^16 = 0,73 "Hz"$. Esta resolución es alta para las notas agudas, pero no es suficiente para las notas
graves. En la @tabla_errores_cents se puede ver el error en _cents_ (una centésima del semitono temperado) con varias
notas.

#include "../../tables/tabla_errores_cents.typ"

Para que el error $e_"cents"$ sea imperceptible, tiene que ser menor a $6 "cents"$ @ref_thesis_minimum_cents. Si
queremos un error $f_e$ imperceptible para la frecuencia $f$ de la cuerda más grave de un bajo de 5 cuerdas, de unos
$30 "Hz"$, el error tiene que ser de $f_e = (2^(1/1.200))^(e_"cents") times f - f approx 0.10 "Hz"$. Dado que
$f_s / 2^"bits" = f_e$, hacen falta $log_2(f_s / f_e) = log_2((48.000 "Hz") / (0.10 "Hz")) = 19$ bits de precisión. Pero
ya que la familia ARM Cortext M es de 32 bits, si se usan más de 16 bits lo mejor es usar 32 (en formato UQ8.24).

Los 8 bits de la parte entera, $i$, se usan directamente para indexar la tabla $T$. De los 24 bits de la parte
fraccionaria, se toman los primeros 15 y se interpretan como un Q15, $r$. Estos se usan para interpolar la muestra $i$
con la siguiente, $i + 1 mod 256$. Interpolar evita el ruido que genera redondear del índice @ref_book_music_tutorial.
Mientras mayor es $r$, más cerca se está de la siguiente muestra, por lo que se usa como el peso de la siguiente
muestra. Por lo tanto, la salida se calcula con la @eq_salida_interpolada @ref_book_theory_music.

$
  T[i] times (1 - r) + T[i+1 mod 256] times r
$
<eq_salida_interpolada>

#let fn = footnote[La idea original era hacerlo con evaluación `const`, que se ejecuta en compile-time
  @ref_web_const_eval. Sin embargo, no fue posible, ya que la mayoría de operaciones con floats no se pueden realizar en
  `const` al no dar los mismos resultados en cualquier CPU @ref_web_powf_not_const.]

Sparklet incluye 4 tablas para los cuatro tipos de ondas que indica el @rf_waveforms. Cada una tiene 256 muestras de 16
bits cada una (en formato Q15), por lo que ocupan 512 bytes cada una y 2 KiB en total. Estas tablas se generan en la con
utilidades que se ejecutan en la computadora de desarrollo con la arquitectura `x86_64` antes del desarrollo. Se
encuentran en la carpeta `sparklet/table_generators/src/bin`.#fn Calculan el valor de cada muestra en la tabla usando
números de coma flotante, los convierten a Q15 y generan archivos de código Rust que definen las tablas.
