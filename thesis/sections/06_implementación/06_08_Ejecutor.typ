#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Ejecutor

El ejecutor (`runner`) es responsable de inicializar el hardware, definir los canales que usan los otros módulos para
comunicarse, y crear las tareas que llaman el resto de componentes. Es el primer módulo mencionado hasta el momento que
sólo se puede ejecutar en el sistema empotrado. En concreto, hace lo siguiente:

+ Inicialización del hardware El módulo `hardware` es responsable de controlar el hardware, es decir las entradas y
  salidas GPIO, EXTI, etc. necesarias. Aislarlo en un módulo facilita actualizar los detalles si se cambia el chip
  usado. Inicializa el USB, si es necesario, y determina los pines que serán usados para los botones y codificadores
  rotativos de la configuración. Finalmente, introduce toda la configuración en el struct `Hardware`, y lo devuelve.

+ Inicialización del ejecutor de Embassy.

+ Inicialización del USB, si está activado.

+ Creación, sin inicializar, de la tarea MIDI, ya sea usando un conector DIN o USB.

+ Creación de la tarea de salida audio por USB.

+ Creación de la tarea de la gestión de la configuración.

+ Creación de la tarea del motor de síntesis.

+ Inicialización de la tarea MIDI, de configuración, de los botones y encoders rotativos, del motor de sínstesis, y de
  la salida de audio.

Una vez acaba, las tareas toman control del chip. La configuración se ejecuta cada cierto tiempo, y la generación de
audio se ejecuta únicamente cuando se pide una salida de audio.
