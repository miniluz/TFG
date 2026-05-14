#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== USB

La manera más fácil de usar Sparklet es por una conexión USB. Sparklet permite usar USB para la salida de audio, según
el @rf_audio_usb, y la entrada de MIDI según el @rf_midi_usb, permitiendo que opere con una única conexión a cualquier
ordenador moderno.

Sparklet usa `embassy_usb` para gestionar la conexión, que abstrae la mayoría de la complejidad. `embassy_usb`
proporciona la implementación de una interfaz para recibir MIDI y una interfaz para ser una entrada de audio (como un
micrófono), por lo que sencillamente se añaden a la descripción del dispositivo las interfaces usadas
@ref_web_usb_audio.

El soporte de audio por USB configura la transmisión de audio para que sea síncrona. En este modo, el dispositivo
maestro pide cada milisegundo un bloque de audio al dispositivo esclavo @ref_web_usb_audio. Se usa para ahorrar la
complejidad de tener un reloj interno independiente y de mantenerlo sincronizado con el dispositivo maestro. Además, se
soporta controlar el volumen y el silenciamiento del sintetizador con señales USB. Cuando se reciben estos eventos, se
almacena el factor equivalente en amplitud y se almacena en el estado, que a su vez se multiplica por el bloque de audio
antes de transmitirlo.
