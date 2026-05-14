// === Parámetros ===
// Onda sinusoidal que alcanza +1 y -1 sobre 11 muestras
#let samples = (0.5, 0.8, 1.0, 0.8, 0.3, -0.7, -1.0, -0.8, -0.3)

// Colores de los rectángulos
#let bar-stroke-color = rgb("#ff7f0e")
#let bar-fill-color = rgb("#ffe0c2")

// Colores del patrón de cebra
#let zebra-even = rgb("#e8e8e8") // empieza por gris
#let zebra-odd = white

// === Dimensiones ===
#let bar-width = 1.05 // 0.6 × 1.75
#let bar-gap = 0.3
#let step = bar-width + bar-gap
#let y-scale = 3 // unidades CeTZ por unidad de amplitud
#let box-stroke = 2pt
#let bar-stroke-w = 1.75pt // 1.5pt × 1.75

#import "@preview/cetz:0.5.2": canvas, draw

#canvas({
  import draw: *

  let n = samples.len()
  let total-w = n * step

  let pad-l = 0.4
  let pad-r = 0.4
  let pad-t = 0.4
  let pad-b = 0.4
  let ox = 1.8

  let box-x0 = ox - pad-l
  let box-x1 = ox + total-w + pad-r
  let box-y0 = -(y-scale + pad-b)
  let box-y1 = (y-scale + pad-t)

  // Centro de barra i
  let cx = i => ox + i * step + bar-width / 2

  // ── Patrón de cebra (cubre la barra más la mitad del gap a cada lado) ──────
  let half-gap = bar-gap / 2
  for i in range(n) {
    let x0 = ox + i * step - half-gap
    let x1 = ox + i * step + bar-width + half-gap
    // recortar al interior de la caja
    let zx0 = calc.max(x0, box-x0)
    let zx1 = calc.min(x1, box-x1)
    let fill = if calc.rem(i, 2) == 0 { zebra-even } else { zebra-odd }
    rect((zx0, box-y0), (zx1, box-y1), stroke: none, fill: fill)
  }

  // ── Caja exterior (encima del zebra) ──────────────────────────────────────
  rect(
    (box-x0, box-y0),
    (box-x1, box-y1),
    stroke: box-stroke + black,
    fill: none,
  )


  // ── Barras ────────────────────────────────────────────────────────────────
  for i in range(n) {
    let amp = samples.at(i)
    let x0 = ox + i * step
    let x1 = x0 + bar-width
    let ry0 = calc.min(0.0, amp * y-scale)
    let ry1 = calc.max(0.0, amp * y-scale)
    rect(
      (x0, ry0),
      (x1, ry1),
      stroke: bar-stroke-w + bar-stroke-color,
      fill: bar-fill-color,
    )
  }

  // ── Línea central (amplitud 0), sobresale a la izquierda ──────────────────
  line(
    (box-x0, 0),
    (box-x1, 0),
    stroke: bar-stroke-w + black,
  )

  let tick-overhang = 0.25
  line(
    (box-x0 - tick-overhang, 0),
    (box-x0, 0),
    stroke: black,
  )

  // ── Etiquetas bajo la caja ────────────────────────────────────────────────
  let label-y = box-y0 - 0.25

  content((cx(0), label-y), [0], anchor: "north")
  content((cx(1), label-y), [1], anchor: "north")
  content((cx(2), label-y), [...], anchor: "north")
  content((cx(n - 1), label-y), [L], anchor: "north")

  content(
    ((box-x0 + box-x1) / 2, label-y),
    [Muestras],
    anchor: "north",
  )

  // ── Eje Y: marcas y etiquetas ─────────────────────────────────────────────
  let elabel-x = box-x0 - 0.4

  // ±1: sobresalen a ambos lados del borde izquierdo únicamente
  for (amp, lbl) in ((1.0, [1]), (-1.0, [-1])) {
    let y = amp * y-scale
    line(
      (box-x0 - tick-overhang, y),
      (box-x0 + tick-overhang, y),
      stroke: black,
    )
    content((elabel-x, y), lbl, anchor: "east")
  }
  // Etiqueta 0
  content((elabel-x, 0), [0], anchor: "east")

  // "Amplitud" rotado
  content(
    (elabel-x - 0.65, 0),
    angle: 90deg,
    [Amplitud],
    anchor: "center",
  )
})
#v(6pt)
