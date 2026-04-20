#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Ejecutor

El ejecutor (`runner`) es el primer módulo mencionado hasta el momento que sólo se puede ejecutar en el sistema
empotrado. Su responsabilidad es inicializar el hardware, definir los canales que usan para comunicarse, y crear las
tareas que llaman al resto de componentes. En la práctica, hace lo siguiente:

+ Inicialización del hardware El módulo `hardware` es responsable de controlar el hardware, es decir los inputs y
  outputs GPIO u EXTI necesarios. Aislarlo en un módulo facilita actualizar los detalles si se cambia el chip usado.
  Inicializa el USB, si es necesario, y determina los pines que serán usados para los botones y codificadores rotativos
  de la configuración. Finalmente, introduce toda la configuración en el struct `Hardware`, y lo devuelve.

+ Inicialización del ejecutor de Embassy.

+ Inicialización del USB, si está activado.

+ Creación, sin inicializar, de la tarea MIDI, ya sea recibido por un conector DIN por UART o por USB. Ésta tarea
  sencillamente lee los datos que se envían por la conexión serial y los envía a un `MidiListener` que contiene en su
  estado.

+ Creación de la tarea de salida audio por USB. Esta espera a que el USB la sondee y devuelve los datos. También
  controla las señales de silenciamiento y control de volumen.

+ Creación de la tarea de la gestión de la configuración.

+ Creación de la tarea del motor de síntesis.

+ Inicialización de la tarea MIDI, de configuración, de los botones y encoders rotativos, del motor de sínstesis, y de
  la salida de audio.

Una vez acaba, el funcionamiento del sistema está únicamente gobernado por sus tareas, que a su vez dependen únicamente
de interrupciones externas. La configuración se ejecuta únicamente cuando se presiona un botón o mueve un codificador, y
el audio se genera únicamente cuando se sondea, envía, y libera un espacio en el canal.
