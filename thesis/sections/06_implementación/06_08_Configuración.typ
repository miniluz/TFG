#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Configuración
<sec_configuración>

Sparklet se configura con dos botones y tres codificadores rotatorios. Para permitir modificar más de tres parámetros,
se usa paginación: los codificadores rotatorios modifican los valores de la página actual, mientras que los botones
controlan la página. El módulo responsable de gestionarlo es `ConfigManager`, que mantiene el estado de los parámetros y
la página seleccionada y procesa eventos de configuración.

Cuando hay una actualización, `ConfigManager` la escribe a un `TripleBuffer` y la publica. Esto permite que
`ConfigManager` nunca se bloquee al escribir y que el hilo de generación de audio nunca se bloquee al leerla.

`ConfigManager` es una tarea independiente a la generación de audio. Está conectada por un `embassy_sync::channel` a las
tareas que leen el estado de los botones y codificadores, de la misma manera que `MidiListener` está conectada a
`Generator`. Las tareas de los botones y codificadores también descartan eventos si la cola está llena; la cola tiene un
tamaño de 32.

/* TODO  Acabar después de reescribir. */

/* TODO Añadir generación de tablas */

/* TODO Documentar cómo se propaga la configuración */
