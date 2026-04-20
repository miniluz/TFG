#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Motor de síntesis

El motor de síntesis es un componente simple que integra el generador con el banco de filtros. Es la capa exterior del
sistema de audio, pero realmente es muy simple. Únicamente provee una interfaz sencilla que inicializa todos sus
componentes y abstrae su funcionamiento.
