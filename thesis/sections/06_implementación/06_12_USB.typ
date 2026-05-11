#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== USB

La manera más fácil de usar el sintetizador es por una conexión USB. Sparklet permite usar USB para la salida de audio y
la entrada de MIDI, permitiendo que opere con una única conexión a cualquier ordenador moderno.

Sparklet usa `embassy_usb` para gestionar la conexión, que abstrae la mayoría de la complejidad. `embassy_usb` provee la
implementación de una interfaz para recibir MIDI y una interfaz para ser una entrada de audio (como un micrófono), por
lo que sencillamente se añaden a la descripción del dispositivo las interfaces usadas @ref_web_usb_audio.

La implementación de USB con MIDI sencillamente lee los datos que se envían y los transmite a `MidiListener`, eliminando
la parte del protocolo que es específica a USB.

La implementación de audio por USB usa el esquema de audio síncrono, en el que el dispositivo maestro pide cada
milisegundo datos al dispositivo @ref_web_usb_audio. De esta manera, Sparklet se ahorra la complejidad de tener un reloj
interno independiente y de mantenerlo sincronizado con el dispositivo maestro. Además, se soporta controlar el volumen y
el silenciamiento del sintetizador con señales USB. Cuando se reciben estos eventos, se almacena en el estado y
sencillamente se multiplica por el volumen correspondiente.
