#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== MIDI

Para gestionar la entrada MIDI, existe el struct `MidiListener`. Expone un método `process_bytes` que recibe un vector
de bytes y lo procesa usando la biblioteca `midly`. `midly` permite identificar mensajes MIDI recibiendo un byte a la
vez, lo que la hace compatible con leer MIDI usando UART.

Cuando `midly` identifica un evento MIDI, `MidiListener` lo envía por el canal de eventos. El canal es un
`embassy_sync::Channel`, _single-sender single-consumer_. Si la cola está llena, el mensaje se descarta. Es fácil
demostrar que la cola de 16 eventos que usa Sparklet es suficiente. Ya que los mensajes se procesan cada milisegundo,
Sparklet puede procesar hasta $16.000$ eventos por segundo; MIDI por UART transmite $31.250$ bits por segundo, y ya que
los mensajes que soporta Sparklet ocupan como mínimo 3 bytes, sólo puede producir $31.250 div 24 approx 1302$ mensajes
por segundo.
#footnote[MIDI por USB puede usar velocidades de transmisión superiores, pero bajo un uso normal no es un problema.]

`Sparklet` únicamente soporta los eventos MIDI `NoteOn` y `NoteOff`. El resto de eventos son descartados por
`MidiListener` antes de enviarlos por el canal. Ambos eventos caben en 3 bytes @ref_web_midi, pero se asigna a `midly`
un buffer de 4 bytes al no tener coste adicional por alineación de memoria. Los mensajes más largos son ignorados al no
caber en el buffer, lo que es conveniente pues MIDI acepta mensajes de longitud arbitraria /* TODO cita */.

La fiabilidad del módulo de `midly` es fundamental, pues es el único módulo expuesto a datos externos que ha de ser
escrito a mano. Se puede encontrar con mensajes erróneos, con ruido, o en el peor caso malicioso. Por lo tanto, éste fue
el módulo más probado. Su resistencia a errores y mensajes largos fue validada: es capaz de procesar mensajes después de
ser alimentada mil bytes de datos aleatorios
