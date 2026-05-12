#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Banco de voces
<sec_banco_de_voces>

Un sintetizador generalmente es capaz de reproducir más de una nota a la vez, lo que se conoce como polifonía
@ref_book_theory_music. Generalmente tienen un límite de polifonía $|V|$. Por ejemplo, si puede reproducir cuatro notas
simultáneamente, se dice que tiene un límite de polifonía de cuatro, o que tiene cuatro voces. El componente de Sparklet
que mantiene el estado de las voces y las gestiona se llama el banco de voces, denominado `VoiceBank`.

El comportamiento del banco de voces es simple si nunca se tocan más de $|V|$ notas simultáneamente: cada vez que se
toca una nota, se busca una voz `Voice` libre (con ADSR en estado `Idle`) y se toca en ella, y cada vez que se deja de
tocar una voz, se pone su `ADSR` respectivo en el estado `Release`.

=== Casos límite de la superación del límite de polifonía

Un sintetizador, para resultar útil a un músico, debe responder de forma predecible e intuitivamente a sus acciones.
Cuando un músico toca una nota, espera oírla, aún si supera el límite de voces; el sintetizador por lo tanto ha de
liberar una voz para tocarla (_voice stealing_) @ref_book_theory_music. La voz no puede parar su nota actual
inmediatamente, pues se oiría un clic, pero ha de liberarse lo antes posible, ya que el músico espera oír la nota que
tocó. La lógica que determina cuál voz liberar se denomina el algoritmo de asignación de voces. Por este motivo, el
envolvente ADSR tiene un modo `QuickRelease`, para soltar una nota inmediatamente incluso con un decaimiento alto.

#cite(<ref_book_theory_music>, form: "prose") propone las bases de un algoritmo simple, pero no es suficiente para
resolver todas las situaciones límite que han de ser manejadas correctamente para que el sintetizador resulte intuitivo
a un músico. Realizar una implementación elegante y eficiente que maneje correctamente todas estas circunstancias fue
una de las dificultades principales del desarrollo de Sparklet. Tómense los siguientes casos:

+ Un músico toca una pieza con un límite de polifonía bajo. Arpegia rápidamente un acorde: do, mi, sol, mi, do; de forma
  que un decaimiento alto haría que cuando toque por segunda vez una nota, la primera voz que la contiene aún no haya
  decaído a cero. Una implementación que no tiene esto en cuenta ocuparía una segunda voz para tocar la nota, y cuando
  se llegase al límite de polifonía, empezaría a cortar notas.

+ Se han recibido dos eventos de tocar la misma nota sin un evento de soltarla en medio, por ejemplo debido a un error
  en el cable. Otra implementación podría asignar una segunda voz a esa nota y liberar sólo una si después se recibe un
  evento de soltar la nota. En este caso, sería imposible soltar la otra nota: incluso si el músico vuelve a tocar y
  soltar la misma tecla, se asignará y liberará otra voz.

+ Entre dos procesamientos de eventos MIDI, un músico toca más notas del límite. Por ejemplo, hay un límite de 2 notas,
  ambas ocupadas, y el músico ha tocado 4 notas desde la última vez que se procesaron. Se han de soltar las notas que
  están siendo tocadas para dar espacio a las nuevas, y priorizar las dos más nuevas de las cuatro. Sin embargo, una
  implementación que no tiene esto en cuenta podría procesar las notas en orden cronológico, y por esto tocar las dos
  notas más antiguas, antes de ver las dos notas que faltan y verse obligado a soltarlas al instante, retrasándolas.

+ Del mismo modo, mientras el sistema aún está intentando tocar notas, un músico toca aún más. Con un límite de 2 notas,
  hay ya dos siendo ya soltadas, dos notas pendientes del último procesamiento, y dos que acaba de tocar el músico y no
  han sido procesadas. Similarmente, un sistema que procese las notas en orden cronológico tocaría primero las dos
  pendientes para soltarlas al instante, retrasando las dos notas del músico.

+ Un músico entre dos eventos de procesamiento toca y suelta una nota. Supóngase que hay un límite de una nota, y que
  además de tocar y soltar la nota después toca otra. Una implementación que no tiene esto en cuenta podría, como en el
  caso anterior, tocar la primera nota, luego tener que soltarla, y finalmente tocar la tercera, retrasándola.

=== Solución implementada

Tomando estos casos límite en cuenta, se implementó el siguiente algoritmo, que los gestiona todos de forma eficiente.
El código de Rust que lo implementa se puede ver en el @cod_voice_steal_algorithm.

El primer problema se puede resolver tomando inspiración en el funcionamiento de un piano. Cuando se usa el pedal para
que las cuerdas no se amortigüen, volver a pulsar una tecla no silencia la nota antes de que el martillo vuelva a
tocarla. Para modelar este comportamiento, `Voice` provee el método `retrigger`, en el que la voz vuelve al estado
`Attack` sin modificar su nivel actual `current`, volviendo a darle intensidad. Este comportamiento se ve ilustrado en
la /* TODO */. Haciendo que tocar una nota siempre haga `retrigger` a la voz que la tiene si ya está siendo tocada, se
contribuye a resolver también el segundo problema, pues se garantiza que sólo una voz reproduce cada nota. Si se da el
caso, volver a tocar y soltar la tecla hará que deje de sonar, volviendo al comportamiento esperado.

Para solucionar los otros tres problemas de forma eficiente, se puede usar una cola intermediaria FIFO de longitud igual
a la cantidad de voces $|V|$. Esta cola almacena las |V| notas más recientes que el usuario ha intentado tocar. Los
eventos MIDI procesados en un lote se procesan en dos pasos: primero son añadidos a la cola, y luego los eventos que
permanecen en la cola se ejecutan. De este modo, antes de empezar a liberar voces, se filtran las notas en exceso del
límite, arreglando el tercer y cuarto problema, y eliminan las notas que se sueltan antes de ser asignadas una voz,
arreglando el quinto.

=== Detalles del algoritmo
<sec_detalles_algoritmo_voice_bank>

Ya que este algoritmo es uno de los aspectos más únicos de Sparklet, cabe entrar en detalle de cómo funciona. Cada vez
que se genera un bloque de audio, lo primero que se hace es que `VoiceBank` procese los eventos MIDI pendientes. En
lugar de aplicarlos directamente, usa la cola intermediaria descrita anteriormente. Por cada evento, en orden
cronológico:

+ Si es de soltar una nota (`NoteOff`):
  + Se mueven todas las voces asociadas a esa nota al estado `Release`.
  + Se elimina la nota de la cola.
+ Si es de tocar una nota (`NoteOn`):
  + Se añade la nota a la cola si aún no se encuentra en ella, expulsando la más antigua si la cola está llena.

Posteriormente, intenta tocar todas las notas de esta cola, parando si se ocupan todas las voces antes de acabar. Si
esto ocurre, se calcula el déficit entre el número de notas pendientes y las voces actualmente en estado `QuickRelease`,
y se activa el modo `QuickRelease` en tantas voces como sea necesario para compensar el déficit. De esta manera, hay
tantas voces en estado `QuickRelease` como notas pendientes en la cola. En un evento de procesamiento futuro, estas
voces habrán pasado al estado `Idle`, por lo que estarán libres. Hasta entonces, la cola de notas a tocar mantiene las
notas más recientes que no han sido soltadas.

La heurística usada para elegir cuál voz pasar a estado `QuickRelease` es la siguiente:

+ La voz con menor amplitud cuyo ADSR esté en estado `Release`. Esto da prioridad a las notas donde el cambio se notará
  menos (las que tienen volumen bajo y ya estaban siendo soltadas).
+ La voz tocando la nota más antigua que no esté ya en estado `QuickRelease` o esté `Idle`.

Si no se encuentra ninguna, no se realiza ninguna operación, ya que significa que todas las voces tienen estado
`QuickRelease` o `Idle`.

#figure(
  text(size: 11pt)[#raw(
    read("/code/voice_steal_algorithm.rs"),
    block: true,
    lang: "rust",
  )],
  caption: [El algoritmo de procesado de eventos MIDI, extraído del método `Generator::render_samples`.],
)<cod_voice_steal_algorithm>
