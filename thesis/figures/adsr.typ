#import "@preview/cetz:0.5.2": canvas, draw

#canvas({
  import draw: *

  // Axis lengths
  let ax-x = 9.5
  let ax-y = 4.5

  // Key x-positions
  let x-a = 1.8 // end of Attack
  let x-d = 3.4 // end of Decay
  let x-s-end = 5.6 // end of Sustain (note released)
  let x-r = 8.5 // end of Release

  // Key y-positions
  let y-max = 4.0 // Amp_max
  let y-sus = 1.8 // Sustain level

  let color_S = rgb("#2ca02c")
  let color_D = rgb("#ff7f0e")
  let color_A = rgb("#1f77b4")
  let color_R = rgb("#9467bd")

  // ── Envelope shape (grey, behind) ──────────────────────────────────────
  set-style(stroke: (paint: color_A, thickness: 3pt))
  line((0, 0), (x-a, y-max))
  set-style(stroke: (paint: color_D, thickness: 3pt))
  line((x-a, y-max), (x-d, y-sus))
  set-style(stroke: (paint: color_S, thickness: 3pt))
  line((x-d, y-sus), (x-s-end, y-sus))
  set-style(stroke: (paint: color_R, thickness: 3pt))
  line((x-s-end, y-sus), (x-r, 0))


  // ── Amp_max dashed line ────────────────────────────────────────────────
  let arrow-y = y-max + 0.55

  set-style(stroke: (paint: gray.darken(10%), thickness: 0.8pt, dash: "dashed"))
  line((0, y-max), (ax-x, y-max))

  // ── Segment delimiters (vertical lines) ───────────────────────────────
  // Attack end / Decay start
  set-style(stroke: (paint: gray, thickness: 0.8pt))
  line((x-a, 0), (x-a, arrow-y))

  // Decay end / Sustain start
  line((x-d, 0), (x-d, arrow-y))

  // Note released
  line((x-s-end, 0), (x-s-end, arrow-y))

  // Release end
  line((x-r, 0), (x-r, arrow-y))

  // ── Sustain vertical arrow (blue) ───────────────────────────────────
  set-style(
    stroke: (paint: color_S, thickness: 2pt, dash: none),
    fill: color_S,
    mark: (end: none, start: ">"),
  )
  line((x-d + 1.1, y-sus), (x-d + 1.1, 0))

  // ── Segment arrows (horizontal) ───────────────────────────────────────

  // A arrow
  set-style(
    stroke: (paint: color_A, thickness: 1.8pt),
    fill: color_A,
    mark: (start: ">", end: ">"),
  )
  line((0, arrow-y), (x-a, arrow-y))

  // D arrow
  set-style(stroke: (paint: color_D, thickness: 1.8pt), fill: color_D)
  line((x-a, arrow-y), (x-d, arrow-y))

  // R arrow
  set-style(stroke: (paint: color_R, thickness: 1.8pt), fill: color_R)
  line((x-s-end, arrow-y), (x-r, arrow-y))

  // ── Axes ───────────────────────────────────────────────────────────────
  set-style(
    stroke: (paint: black, thickness: 1.8pt),
    fill: black,
    mark: (start: none, end: ">"),
  )

  // x-axis
  line((0, 0), (ax-x, 0), mark: (end: ">", fill: black))
  // y-axis
  line((0, 0), (0, ax-y), mark: (end: ">", fill: black))

  // ── Labels ─────────────────────────────────────────────────────────────
  set-style(stroke: none, fill: black)

  // Segment letters
  content(((0 + x-a) / 2, arrow-y + 0.45), text(
    fill: color_A,
    size: 18pt,
    style: "italic",
    weight: "bold",
  )[A])
  content(((x-a + x-d) / 2, arrow-y + 0.45), text(
    fill: color_D,
    size: 18pt,
    style: "italic",
    weight: "bold",
  )[D])
  content((x-d + 1.1, y-sus + 0.45), text(
    fill: color_S,
    size: 18pt,
    style: "italic",
    weight: "bold",
  )[S])
  content(((x-s-end + x-r) / 2, arrow-y + 0.45), text(
    fill: color_R,
    size: 18pt,
    style: "italic",
    weight: "bold",
  )[R])

  // Axis labels
  content((ax-x * 0.35, -0.5), text(size: 14pt)[Tiempo])
  content((-0.4, ax-y * 0.5), angle: 90deg, text(size: 14pt)[Amplitud])

  // Origin
  content((-0.25, -0.3), text(size: 14pt, weight: "bold")[0])

  // x-axis annotations
  content((0.5, -0.4), text(size: 8.5pt)[nota\ tocada])
  content((x-s-end + 0.5, -0.4), text(size: 8.5pt)[nota\ soltada])
})
