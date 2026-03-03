#import "@preview/deal-us-tfc-template:1.0.0": *

#show: TFC.with(
  titulo: // cspell:disable-line
  "Síntesis en Rust",
  alumno: "Javier Ignacio Milá de la Roca Dos Santos",
  titulacion: // cspell:disable-line
  "Grado en Ingeniería Informática - Ingeniería del Software",
  director: [Director 1 \ Director 2],
  departamento: "Departamento de Tecnología Electrónica",
  convocatoria: "Convocatoria de junio, curso 2025/26",
  dedicatoria: "Aquí la dedicatoria del trabajo",
  agradecimientos: [
    Quiero agradecer a Z por...

    También quiero agradecer a Y por...
  ],
  resumen: [
    Incluya aquí un resumen de los aspectos generales de su trabajo, en español
  ],
  palabras-clave: (
    "palabra clave 1",
    "palabra clave 2",
    "...",
    "palabra clave N",
  ),
  abstract: [
    This section should contain an English version of the Spanish abstract.
  ],
  keywords: (
    "keyword 1",
    "keyword 2",
    "...",
    "keyword N",
  ),
  font: "TeX Gyre Pagella",
)

#include "sections/ejemplos_bórrame.typ"
#include "sections/01_introducción.typ" // cspell:disable-line
#include "sections/02_Gestión.typ"
#include "sections/03_Análisis.typ"
#include "sections/04_Diseño.typ"
#include "sections/05_Implementación.typ"
#include "sections/06_Pruebas.typ"
#include "sections/XX_Conclusiones.typ"

#bibliography("/bibliografía.bib")
