#let req-ids = state("req-ids", (:))
#let req-counter = counter("reqs")

#let req(lbl, prefix, short, body) = {
  req-counter.step()
  context {
    let id = prefix + req-counter.display()
    req-ids.update(d => {
      d.insert(lbl, (id, short))
      d
    })
    [#figure(
        kind: "req",
        supplement: [REQ],
        grid(
          columns: (auto, 1fr),
          column-gutter: 1em,
          align(top, pad(left: 1em, [R#id.])),
          align(top + left, [#short: #body]),
        ),
      ) #label(lbl)]
  }
}

#let setup-reqs(doc) = {
  show ref: it => context {
    if (
      it.element != none
        and it.element.func() == figure
        and it.element.kind == "req"
    ) {
      let (id, short) = req-ids.final().at(str(it.target), default: "?")
      link(it.target, text(fill: black, [R#id (#short)]))
    } else { it }
  }

  doc
}
