#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Banco de voces
<sec_banco_de_voces>

Un sintetizador generalmente es capaz de reproducir más de una nota a la vez, lo que se conoce como polifonía
@ref_book_theory_music. Generalmente tienen un límite de polifonía $|V|$ . Por ejemplo, si puede reproducir cuatro notas
simultáneamente, se dice que tiene un límite de polifonía de cuatro, o que tiene cuatro voces. El componente de Sparklet
que mantiene el estado de las voces y las gestiona se llama el banco de voices, `VoiceBank`.

Sparklet modela las voces `Voice` como máquinas de estado simples, con únicamente dos estados `VoiceStage`: libre
(`Free`) y sostenida (`Held`). El comportamiento del banco de voces es simple si nunca se tocan más de $|V|$ notas
simultáneamente: cada vez que se toca una nota, se busca una voz libre, y cada vez que se deja de tocar una voz, se pone
su `ADSR` respectivo en el estado `Release`, y una vez la amplitud llega a 0, se libera (se pasa a `Free`).

Una voz `Voice` almacena:
- el número de la muestra en el que fue tocada (`timestamp`),
- la nota que fue tocada (`note`) y su velocidad (`velocity`),
- el ADSR de dicha nota (`adsr`)
- y su oscilador (`wavetable_osc`).

Una voz es simple, sencillamente expone funciones para tocar una nota (`play_note`, `retrigger`).

Un banco de voces `VoiceBank` sencillamente tiene:
- un vector de `Voice` (`voices`) y
- el contador de muestras global (`timestamp_counter`).

=== Casos límite de la superación del límite de polifonía

Un sintetizador útil para un músico ha de responder intuitivamente a sus acciones. Cuando un músico toca una nota,
espera oírla, aún si supera el límite de voces; el sintetizador por lo tanto ha de liberar una voz para tocarla. La voz
no puede parar su nota actual inmediatamente, pues se oiría un clic, pero ha de liberarse lo antes posible, ya que el
músico espera oír la nota que tocó. La lógica que determina qué voz liberar y cómo se llama el algoritmo de distribución
de voces. Es por esto que el envolvente ADSR tiene un modo `QuickRelease`, para solar una nota inmediatamente incluso
con un decaimiento alto.

#cite(<ref_book_theory_music>, form: "prose") propone las bases de un algoritmo simple, pero no es suficiente para
resolver todas las situaciones límite que han de ser manejadas correctamente para que el sintetizador resulte intuitivo
a un músico. Realizar una implementación elegante y eficiente que maneje correctamente todas estas circunstancias fue
una de las dificultades principales del desarrollo de Sparklet. Tómense los siguientes casos:

+ Un músico toca una pieza con un límite de polifonía bajo. Arpegia rápidamente un acorde: do, mi, sol, mi, do; de forma
  que un decaimiento alto haría que cuando toque por segunda vez una nota, la primera voz que la contiene aún no haya
  decaído a cero. Una implementación que no tiene esto en cuenta ocuparía una segunda voz para tocar la nota, y cuando
  se llegase al límite de polifonía, empezaría a cortar notas. Pero un piano no funciona así: cuando se usa el pedal
  para que las cuerdas no se amortigüen, volver a pulsar una tecla no silencia la nota antes de que el martillo vuelva a
  tocar la cuerda. Por esto, `Voice` provee el método `retrigger`, en el que la voz vuelve al estado `Attack` sin
  modificar su nivel actual `current`, efectivamente volviendo a darle intensidad como se ve en la /* TODO */.

+ Se han recibido dos eventos de tocar la misma nota sin un evento de soltarla en medio, por ejemplo debido a un error
  en el cable. Otra implementación podría asignar una segunda voz a esa nota y liberar sólo una si después se recibe un
  evento de soltar la nota, haciendo imposible soltar la otra. La implementación de Sparklet verifica que cada nota sólo
  se reproduce en una voz.

+ Un músico toca entre dos procesamientos de la cola más notas del límite. Por ejemplo, hay un límite de 2 notas, que ya
  están ambas voces ocupadas, y el músico ha tocado 4 notas desde la última vez que se procesaron. Claro, hay que soltar
  las notas que están siendo tocadas para dar espacio a las nuevas, y priorizar las dos más nuevas de las cuatro. Sin
  embargo, una implementación que no tiene esto en cuenta podría procesar las notas en orden cronológico, y por esto
  tocar las dos notas más antiguas de la cola, antes de ver las dos notas que faltan y verse obligado a soltarlas al
  instante, retrasándolas. Por esto, se implementa una cola intermediaria para las notas a tocar, de longitud igual a la
  cantidad de voces total, $|V|$.

+ Similarmente, mientras el sistema aún está intentando tocar notas, un músico toca aún más. Con un límite de 2 notas,
  hay ya dos siento ya soltadas, dos notas pendientes del último procesamiento, y dos que acaba de tocar el músico y no
  han sido procesadas. Similarmente, un sistema que procese las notas en orden cronológico tocaría primero las dos
  pendientes para soltarlas al instante, retrasando las dos notas del músico. La misma cola intermediaria del paso
  anterior soluciona este problema.

+ Un músico entre dos eventos de procesamiento toca y suelta una nota. Supóngase que hay un límite de una nota, y que
  además de tocar y soltar la nota después toca otra. Una implementación que no tiene esto en cuenta podría, como en el
  caso anterior, tocar la primera nota, luego tener que soltarla, y finalmente tocar la tercera, retrasándola. Por esto,
  cuando se recibe un evento de soltar una nota, aparte de indicarlo a la voz que tiene asignada esa nota, elimina de la
  cola de notas a tocar, abriendo espacio para otras.

=== Detalles del algoritmo
<sec_detalles_algoritmo_voice_bank>

Tomando estos casos límite en cuenta, se implementó el siguiente algoritmo, que los gestiona todos de forma eficiente.
El código de Rust que lo implementa se puede ver en el @cod_voice_steal_algorithm.

El método `play_note` de `VoiceBank` nunca toca una nota si no quedan voces libres. Por voz libre, se entiende una voz
en estado `Idle`, sin contar `Release` ni `QuickReleaes`. En su lugar, `play_note` devuelve `AllVoicesBusy`. La
responsabilidad de liberar una voz y volverlo a intentar más tarde recae en el módulo que llama al `VoiceBank`. Para
liberarla, se usa el método `quick_release` de `VoiceBank`, que activa el modo `QuickRelease` del ADSR de una de sus
notas, eligiendo con la siguiente prioridad:

+ La voz más callada cuyo ADSR esté en estado `Release`. Ésto da prioridad a las notas donde el cambio se notará menos
  (las que tienen volumen bajo y ya estaban siendo soltadas)
+ La voz tocando la nota más antigua que no esté ya en estado `QuickRelease` o esté `Idle`.

Si no se encuentra ninguna, no se realiza ninguna operación, ya que significa que todas las voces tienen estado
`QuickRelease` o `Idle`. Después de $n$ llamadas, habrán al menos $n$ notas en modo `QuickRelease` o `Idle`. También
proporciona `count_vocies_in_quick_release`, para saber cuántas notas están siendo liberadas.

Aunque el módulo que gestiona la liberación de voces, el generador `Generator`, se explicará en la siguiente sección,
cabe explicar en esta cómo lo hace. Cuando se pide al generador que calcule un bloque de audio, lo primero que hace es
procesar los eventos MIDI de tocar y soltar notas. En lugar de aplicarlos directamente, usa la cola de notas a tocar
descrita anteriormente, que es de tipo FIFO y de longitud `|V|`. Por cada evento, en orden cronológico:

+ Si es de soltar una nota:
  + Pide al `VoiceBank` que mueva todas las voces asociadas a esa nota al estado `Release`.
  + Elimina la nota de la cola.
+ Si es de tocar una nota:
  + Añade la nota a la cola si aún no se encuentra en ella, expulsando la más antigua si la cola está llena.

Posteriormente, intenta tocar todas las notas de esta cola, parando si antes recibe un `AllVoicesBusy`, indicando que no
quedan voces libres. Si es así, calcula la diferencia $d$ entre la cantidad de notas que quedan en la cola y la cantidad
de voces que están ya en modo `QuickRelease`. Ya que `VoiceBank::quick_release` pone una voz más en modo `QuickRelease`
cada vez que se invoca, la llama $d$ veces, para así liberar las voces necesarias para vaciar la cola. En un evento de
procesamiento futuro, estas voces habrán pasado al estado `Idle`, por lo que estarán libres. Hasta entonces, la cola de
notas a tocar mantiene las notas más recientes que no han sido soltadas.

#figure(
  text(size: 11pt)[#raw(
    read("/code/voice_steal_algorithm.rs"),
    block: true,
    lang: "rust",
  )],
  caption: [El algoritmo de procesado de eventos MIDI, extraído del método `Generator::render_samples`.],
)<cod_voice_steal_algorithm>
