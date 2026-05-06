#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Banco de voces
<sec_banco_de_voces>

Un sintetizador generalmente sólo puede reproducir una cierta cantidad de notas a la vez. Esto es lo que se conoce como
un límite de polifonía $|V|$. Por ejemplo, si puede reproducir cuatro notas simultáneamente, se dice que tiene un límite
de polifonía de cuatro, o que tiene cuatro voces. El componente de Sparklet que mantiene el estado de las voces y las
gestiona se llama el banco de voices, `VoiceBank`.

Sparklet modela las voces `Voice` como máquinas de estado simples, con únicamente dos estados `VoiceStage`: libre
(`Free`) y sostenida (`Held`). El comportamiento del banco de voces es simple si nunca se tocan más de $|V|$ notas
simultáneamente: cada vez que se toca una nota, se busca una voz libre, y cada vez que se deja de tocar una voz, se pone
su `ADSR` respectivo en el estado `Release`, y una vez la amplitud llega a 0, se libera (se pasa a `Free`).

Una voz `Voice` almacena:
- el número de la muestra en el que fue tocada (`timestamp`),
- la nota que fue tocada (`note`) y su velocidad (`velocity`),
- el ADSR de dicha nota (`adsr`)
- y su oscilador (`wavetable_osc`).

Una voz es simple, sencillamente expone funciones para tocar una nota (`play_note`)

Un banco de voces `VoiceBank` sencillamente tiene:
- una matriz de voces (`voices`) y
- el contador de muestras global (`timestamp_counter`).

=== Procesamiento de eventos MIDI
<sec_procesamiento_midi_voice_bank>

Los bancos de voces tienen una serie de situaciones límite que manejar. El primero y el más simple es cuando se toca una
nota, se deja de tocar (pasándola al estado `Release`) y se vuelve a tocar antes de que la nota se libere. En ese caso,
se activa un evento `retrigger`, en el que la voz vuelve al estado `Attack` manteniendo el nivel `current` en el que se
encuentra, suavizando tocar y soltar una nota repetidas veces. Una alternativa aceptable sería ocupar otra voz, pero
reutilizarla es preferible cuando el límite de polifonía es bajo. Por ejemplo, con un límite de dos notas, permite
oscilar rápidamente entre las dos notas sin tener que cortar ninguna de las dos.

Un caso con más complejidad es cuando se toca una nota más que el límite. En este caso, una opción sería buscar la voz
tocando la nota más antigua y sobreescribirla repentinamente, pero el salto repentino sería perceptible. Como solución,
`VoiceBank` no toca una nota si no hay una voz libre (aunque el ADSR esté en estado `Release`); en su lugar devuelve un
error `AllVoicesBusy`. La responsabilidad de liberar una nota y volverlo a intentar más adelante recae en el módulo que
llama al `VoiceBank`. Para esto, el `VoiceBank` expone la función `quick_release`, que activa el modo `QuickRelease` del
ADSR de una de sus notas, eligiendo con la siguiente prioridad:

/* TODO: Añadir visualizaciones para esta parte */

+ La voz más callada cuyo ADSR esté en estado `Release`. Ésto da prioridad a las notas donde el cambio se notará menos
  (las que tienen volumen bajo y ya estaban siendo liberadas)
+ La voz tocando la nota más antigua que no esté ya en estado `QuickRelease` o esté `Idle`.

Si no se encuentra ninguna, no realiza ninguna operación. Si no hay ninguna nota en `Idle`, después de $n$ llamadas,
habrán al menos $n$ notas en modo `QuickRelease`. También proporciona `count_vocies_in_quick_release`, para saber
cuántas notas están siendo liberadas.

Aunque se hablará de él a continuación, cabe explicar qué es lo que hace el módulo que llama a `VoiceBank`, el generador
`Generator`. Cuando se le pide que genere los niveles para un grupo de muestras, lo primero que hace es procesar los
eventos MIDI que se corresponden a tocar o dejar de tocar una nota. Procesa los eventos y los almacena en una cola FIFO
de notas a tocar de longitud `|V|` #footnote[Usar una cola FIFO de longitud `|V|` resuelve una situación límite: si
  entre dos eventos de procesamiento se intentan tocar más de `|V|` notas a la vez, se eliminan de la cola las más
  antiguas. Una implementación distinta podría tocarlas por un instante antes de liberarlas.].

+ Si es de liberar una nota:
  + Pide al `VoiceBank` que mueva todas las voces asociadas a esa nota al estado `Release`.
  + Elimina la nota de la cola.
+ Si es de tocar una nota:
  + Añade la nota a la cola, si no está ya en ella, expulsando la más antigua si está llena.

Ya que soltar una nota la elimina de la cola, si entre dos eventos de procesamiento de notas se toca y libera una nota,
no se procesa. Posteriormente, intenta tocar todas las notas de la cola, parando si todas las voces están ocupadas. Si
es así, llama `VoiceBank::quick_release` tantas veces como notas quedan en la cola.

Este sistema maneja correctamente todas las situaciones límite donde se supera el límite de notas de manera eficiente en
términos computacionales.
