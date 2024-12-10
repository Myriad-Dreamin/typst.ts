#set page(height: auto, width: auto, margin: 0pt)

#let s = state("t", (:))

#let pin(t) = context {
  let width = measure(line(length: here().position().y)).width
  s.update(it => it.insert(t, width) + it)
}

#show math.equation: it => {
  box(it, inset: (top: 0.5em, bottom: 0.5em))
}

$pin("l1")1+(e vec(a,b,c)) / sqrt(sqrt(a/c)/(e + c +a/b)/e)$ 123

#context [
  #metadata(s.final().at("l1")) <label>
]

// #context s.get()

#context {
  let s = s.final()
  place(left + top, dx: 0pt, dy: s.l1, line(length: 100pt, stroke: red + 0.1pt))
}
