#set page(height: auto, width: auto, margin: 0pt)

#let s = state("t", (:))

#let pin(t) = locate(loc => {
  style(styles => s.update(it => it.insert(t, measure(line(length: loc.position().y + 0.25em), styles).width) + it))
})

#show math.equation: it => {
  box(it, inset: (top: 0.5em, bottom: 0.5em))
}

$pin("l1")1+e/sqrt(sqrt(a/c)/(e + c +a/b))$

#locate(loc => [
  #metadata(s.final(loc).at("l1")) <label>
])

// #s.display()
// #locate(loc => {
//   let s = s.final(loc)
//   place(left+top, dx: 0pt, dy: s.l1, line(length: 100pt, stroke: red + 0.1pt))
// })
