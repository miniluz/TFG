#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== MIDI

Para gestionar la entrada MIDI, existe el struct `MidiListener`. Este expone un método `process_bytes` que recibe una
matriz de bytes y los procesa usando la librería `midly`. El stream puede contener mensajes incompletos -- `midly`
mantiene un almacenamiento interno para completarlos. Cuando `midly` identifica un evento MIDI, `MidiListener` lo filtra
y únicamente lo comunica a la cola de eventos MIDI (que recibe `Generator`) si es uno de los que `Sparklet` puede
procesar.

`Sparklet` únicamente procesa eventos `NoteOn` and `NoteOff`. Ya que MIDI puede tener eventos de longitud arbitraria
debido a su extensibilidad, y ya que ambos eventos caben en 4 bytes, para ahorrar memoria se usa un buffer de 4 bytes
para `midly`. Los mensajes más largos son ignorados. Las pruebas unitarias validan este comportamiento.
