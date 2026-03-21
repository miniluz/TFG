#import "@preview/deal-us-tfc-template:1.0.0": *

#show math.equation.where(block: true): set text(14pt)

== Derivación matemática de las fórmulas de ASDR
<sec_derivación_asdr>

Se buscan derivar las siguientes tres ecuaciones en base al comportamiento de un condensador:

#context query(<eq_decay_b_c>).first()

#context query(<eq_base_coefficient_t_r>).first()

#context query(<eq_b_c_range>).first()

La fórmula que modela el comportamiento de un condensador cuando se cambia el voltaje entre sus terminales es la decaída
exponencial. Cuando un condensador tiene un voltaje $y_0$ y se cierra su circuito con una resistencia, su voltaje sobre
el tiempo $y(t)$ decae a 0, siguiendo la @eq_decay_to_0, donde $tau$ determina la duración que toma la decaída (a mayor
$tau$, mayor duración).

$
  y(t) = y_0 times e^inline((-t)/tau)
$
<eq_decay_to_0>

Cuando el condensador cambia de un valor inicial $y_0$ a un valor objetivo $T$, sigue la @eq_decay_to_T:

/* TODO: Replace -t/tau with (-t)/tau, -1/tau with (-1)/tau, -2/tau with (-2)/tau */

$
  y(t) = T + (y_0 - T) times e^inline((-t)/tau)
$
<eq_decay_to_T>

La representación iterativa de esta fórmula continua es la @eq_decay_iterative:

$
  y_n = T + (y_(n-1) - T) times e^inline((-1)/tau)
$
<eq_decay_iterative>

Su equivalencia se puede demostrar con la hipótesis de inducción definida en la @eq_induction_hypothesis. El caso base
$m = 1$ es verdadero por definición.

$
  y_n = T + (y_(n-m) - T) times e^inline((-m)/tau)
$
<eq_induction_hypothesis>

#context query(<eq_decay_iterative>).first()

Y se puede demostrar que si es cierto para $m$, es cierto para $m + 1$, cumpliendo la hipótesis de inducción:

#context query(<eq_induction_hypothesis>).first()

#math.equation(
  [$
    y_(n-m) = T + (y_(n-(m+1)) - T) times e^inline((-1)/tau)
    \
    \
    \
    y_n = T + (T + (y_(n-(m+1)) - T) times e^inline((-1)/tau) - T) times e^inline((-m)/tau)
    \
    y_n = T + (y_(n-(m+1))) times e^inline((-1)/tau) times e^inline((-m)/tau)
    \
    y_n = T + (y_(n-(m+1))) times e^inline((-(m+1))/tau)
  $],
  block: true,
  numbering: none,
)

Por lo tanto, tomando $m = n$, queda demostrado con la @eq_decay_iterative_is_equivalent que la @eq_decay_iterative es
equivalente a la @eq_decay_to_T:

#context query(<eq_decay_iterative>).first()

#context query(<eq_induction_hypothesis>).first()

$
  y_n = T + (y_0 - T) times e^inline((-n)/tau)
$
<eq_decay_iterative_is_equivalent>

#context query(<eq_decay_to_T>).first()

Para simplificar la @eq_decay_iterative, se define el coeficiente $C$ y la base $B$, como se ve en la
@eq_base_coefficient. Esto permite redefinirla en términos de $B$ y $C$, como se ve en la @eq_decay_b_c:

#block(
  [
    $
      C = e^inline((-1)/tau)
      \
      B = T times (1 - C)
    $
    <eq_base_coefficient>

    #math.equation(
      [$
        y_n = T + (y_(n-1) - T) times e^inline((-1)/tau)
        \
        y_n = T + (y_(n-1) - T) times C
        \
        y_n = T + y_(n-1) times C - T times C
        \
        y_n = T times (1 - C) + y_(n-1) times C
      $],
      block: true,
      numbering: none,
    )

    #context query(<eq_decay_b_c>).first()
  ],
  breakable: false,
)

Se podría controlar $tau$ para controlar la velocidad de la decaída. Sin embargo, esto tiene dos inconvenientes: sólo
permitiría tener una curva de decaimiento exponencial hacia un objetivo (es decir, no permite controlar la forma de la
curva), y el decaimiento exponencial nunca llega a su valor objetivo. Esto dificultaría crear la máquina de estados que
modela el envolvente ADSR. La transición del estado de ataque al de decaimiento ocurre cuando se llega al volumen
máximo, y una decaída exponencial nunca llega a su valor objetivo.

Lo que propone #cite(<ref_web_adsr>, form: "prose") es que el objetivo $T$ al que decaer no sea el objetivo $T_0$ al que
se quiere llegar, sino que se exceda por un _target ratio_ $r > 0$. Por ejemplo, si el estado inicial es $y_0 = 1$ y se
quiere llegar al objetivo $T_0 = 0$, el decaimiento se realiza con target ratio $r = 0.5$, el decaimiento se realiza
hacia un valor $T$ un 50% más lejos del objetivo real $T_0$, es decir hacia $T = 1.5$. $T$ se calcula con la @eq_t_r.

$
  T = y_0 + (T_0 - y_0) times (1 + r)
$
<eq_t_r>


Si se consigue el $tau$ necesario para decaer de $y_0$ a $T_0$ en $n$ muestras, se pueden calcular $C$ y $B$. Despejar
$tau$ resulta en el valor indicado en la @eq_tau_t_r:

#context query(<eq_decay_iterative>).first()

#v(0.75em)

#math.equation(
  [$
    y_n = T + (y_0 - T) times e^inline((-n)/tau)
    \
    \
    y_n = T_0
    \
    \
    T = y_0 + (T_0 - y_0) times (1 + r)
  $],
  block: true,
  numbering: none,
)

#v(0.75em)

#math.equation(
  [$
    T_0 = T + (y_0 - T) times e^inline((-n)/tau)
    \
    \
    T_0 = y_0 + (T_0 - y_0) (1 + r) + (y_0 - T) times e^inline((-n)/tau)
    \
    \
    T_0 = y_0 + T_0 (1 + r) - y_0 (1 + r) + (y_0 - T) times e^inline((-n)/tau)
  $],
  block: true,
  numbering: none,
)

#v(0.75em)

#math.equation(
  [$
    T_0 - T_0 (1 + r) = y_0 - y_0 (1 + r) + (y_0 - T) times e^inline((-n)/tau)
    \
    \
    -T_0 r = -y_0 r + (y_0 - T) times e^inline((-n)/tau)
    \
    \
    y_0 r - T_0 r = (y_0 - T) times e^inline((-n)/tau)
    \
    \
    (y_0 - T_0) times r = (y_0 - T) times e^inline((-n)/tau)
  $],
  block: true,
  numbering: none,
)

#v(0.75em)

#math.equation(
  [$
    (y_0 - T_0) times r = (y_0 - y_0 - (T_0 - y_0) times (1 + r)) times e^inline((-n)/tau)
    \
    \
    (y_0 - T_0) times r = (y_0 - T_0) times (1 + r) times e^inline((-n)/tau)
    \
    \
    r = (1 + r) times e^inline((-n)/tau)
  $],
  block: true,
  numbering: none,
)

#v(1em)

#math.equation(
  [$
    e^inline((-n)/tau) = r / (1 + r)
    \
    \
    -n / tau = ln(r / (1 + r))
    \
    \
    tau = -n div ln(r / (1 + r))
  $],
  block: true,
  numbering: none,
)

#v(1em)

$
  tau = n div ln((1 + r) / r)
$
<eq_tau_t_r>

Por lo tanto, dada la @eq_base_coefficient, se redefinen $C$ y $B$ en base al valor inicial $y_0$, el objetivo $T$, el
target ratio $r$ y el la cantidad de muestras que tarda $n$ en la @eq_base_coefficient_t_r:

#block(
  [
    #math.equation(
      [$
        C = e^inline((-1)/tau)
        \
        C = e^(-1 div (n div ln(inline((1 + r) / r))))
        \
        C = e^(ln(inline(r / (1 + r))) div n)
      $],
      block: true,
      numbering: none,
    )

    #context query(<eq_base_coefficient_t_r>).first()
  ],
  breakable: false,
)

/* TODO Insertar código Rust */

Finalmente queda demostrar que pueden ser almacenados en un Q15. Es importante notar que $T_0$ y $y_0$ en la práctica
sólo tienen dos casos:
+ En el attack, $y_0 = 0$ y $T_0 = 1$
+ En el decay y release, $y_0 = 1$ y $T_0 = 0$

Por lo tanto, es suficiente demostrar lo que afirma la @eq_b_c_range:

#context query(<eq_b_c_range>).first()

Es fácil demostrarlo para $C$, ya que por definición $r > 0$:

#math.equation(
  [$
    forall r > 0 : #h(1em) 0 < inline(r / (1 + r)) < 1
    \
    forall r > 0, n > 0: #h(1em) 0 < C = (inline(r / (1 + r)))^(1/n) < 1
  $],
  block: true,
  numbering: none,
)

Para $B$, se pueden evaluar ambos casos de forma independiente:

#context query(<eq_base_coefficient_t_r>).first()

Para el caso $y_0 = 0$, $T_0 = 1$:

#math.equation(
  [$
    T = 0 + (1 - 0)(1 + r) = 1 + r
    \
    \
    B = (1 + r) times (1 - C) = (1 + r) times (1 - (inline(r/(1+r)))^(1/n))
  $],
  block: true,
  numbering: none,
)

El límite inferior se da ya que, si $r > 0$, ya se demostró que $0 < C < 1$, y por lo tanto siempre se cumple que
$(1 + r) times (1 - C) > 0$.

El límite superior, $B = (1+r) times (1 - (inline(r/(1+r)))^(1/n)) <= 1$, se puede demostrar con:

#math.equation(
  [$
    B = (1+r) times (1 - (inline(r/(1+r)))^(1/n)) <= 1
    \
    \
    1 - (inline(r/(1+r)))^(1/n) <= inline(1/(1+r))
    \
    \
    (inline(r/(1+r)))^(1/n) >= 1 - inline(1/(1+r)) = inline(r/(1+r))
    \
    \
    forall r > 0: #h(1em) 0 < inline(r/(1+r)) < 1
    \
    \
    forall r > 0, n >= 1: #h(1em) (inline(r/(1+r)))^(1/n) >= inline(r/(1+r))
  $],
  block: true,
  numbering: none,
)

Para el caso $y_0 = 1$, $T_0 = 0$:

#math.equation(
  [$
    T = 1 + (0 - 1)(1 + r) = 1 - (1 + r) = -r
    \
    \
    B = -r times (1 - C) = -r times (1 - (inline(r/(1+r)))^(1/n))
  $],
  block: true,
  numbering: none,
)

El límite superior se da ya que si $r > 0$ y $0 < C < 1$, $-r times (1 - C) < 0$.

El límite inferior, $-1 <= B = -r times (1 - (inline(r/(1+r)))^(1/n))$, se puede demostrar con:

#math.equation(
  [$
    B = -r times (1 - (inline(r/(1+r)))^(1/n)) >= -1
    \
    \
    r times (1 - (inline(r/(1+r)))^(1/n)) <= 1
    \
    \
    1 - (inline(r/(1+r)))^(1/n) <= inline(1/r)
    \
    \
    -(inline(r/(1+r)))^(1/n) <= inline((1-r)/r)
    \
    \
    (inline(r/(1+r)))^(1/n) >= inline((r - 1)/r)
  $],
  block: true,
  numbering: none,
)

Dado que $forall r > 0, n >= 1 : (r/(1+r))^(1/n) >= r/(1+r)$, para demostrar que $-1 <= B$ es suficiente demostrar que
$forall r > 0 : r/(r+1) >= (r-1)/r$.

#math.equation(
  [$
    forall r > 0 : #h(1em) inline(r/(r+1)) >= inline((r-1)/r)
    \
    \
    forall r > 0 : #h(1em) r^2 >= (r+1)(r-1) = r^2 - 1
  $],
  block: true,
  numbering: none,
)

Queda así demostrada la @eq_b_c_range.

#context query(<eq_b_c_range>).first()
