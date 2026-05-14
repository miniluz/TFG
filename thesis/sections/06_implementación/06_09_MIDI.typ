#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== MIDI

El protocolo MIDI es un estándar para transmitir información de control entre dispositivos musicales @ref_web_midi
@ref_book_music_tutorial. En lugar de transmitir audio, transmite eventos, como tocar o soltar una nota o los mensajes
de _Control Change_ (CC), que regulan 128 parámetros con valores 0-127. Permite también realizar control mucho más
complejo, como la sincronización de tempo entre dispositivos. Un dispositivo MIDI puede ignorar los mensajes que no
soporta @ref_web_midi.

Para gestionar la entrada MIDI, se usa el `struct` `MidiListener`. Expone un método `process_bytes` que recibe un vector
de bytes y lo procesa usando la biblioteca `midly`. `midly` permite identificar mensajes MIDI recibiendo un byte a la
vez, lo que la hace compatible con leer MIDI usando UART. Cuando `midly` identifica un evento MIDI, `MidiListener` lo
envía al `VoiceBank` por el canal de eventos, que los procesa como se explica en la @sec_procesado_midi. El canal es un
`embassy_sync::Channel`, _single-sender single-consumer_. Si la cola de 16 eventos está llena, el mensaje se descarta.
#footnote[Dado que los mensajes se procesan cada vez que se genera audio, cada milisegundo, Sparklet puede procesar
  hasta $16 times 1.000 = 16.000$ eventos por segundo; MIDI por UART transmite $31.250$ bits por segundo, y ya que los
  mensajes que soporta Sparklet ocupan como mínimo 3 bytes, solo puede producir $31.250 div 24 approx 1302$ mensajes por
  segundo. MIDI por USB puede usar velocidades de transmición superiores, pero la cola en la práctica nunca se llena.]


`Sparklet` únicamente soporta los eventos MIDI `NoteOn` y `NoteOff`. El resto de eventos son descartados por
`MidiListener` antes de enviarlos por el canal. Ambos eventos caben en 3 bytes @ref_web_midi, pero se asigna a `midly`
un _buffer_ de 4 bytes al no tener coste adicional por alineación de memoria. Los mensajes más largos son ignorados al
no caber en el buffer, lo que es conveniente pues MIDI acepta mensajes de longitud arbitraria /* TODO cita */.

Sparklet soporta la entrada MIDI tanto por un puerto DIN, usando UART, como por USB, según el @rf_midi_din y el
@rf_midi_usb. En ambos casos, se consiguen los bytes de los mensajes MIDI y se envían a `MidiListener`.

La fiabilidad del módulo de `midly` es fundamental, pues es el único módulo expuesto a datos externos con código de
gestión escrito a mano. Se puede encontrar con mensajes erróneos, con ruido, o en el peor caso maliciosos, y los ha de
manejar correctamente para cumplir el @rnf_fiabilidad. Por lo tanto, este fue el módulo más probado. Su resistencia a
errores y mensajes largos fue validada: es capaz de procesar mensajes tras haber recibido mil bytes de datos aleatorios

// TODO: Hablar de MIDI con UART
