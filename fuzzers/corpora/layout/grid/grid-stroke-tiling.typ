
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let double-line = tiling(size: (1.5pt, 1.5pt), {
  place(line(stroke: .6pt, start: (0%, 50%), end: (100%, 50%)))
})

#table(
  stroke: (_, y) => if y != 1 { (bottom: black) },
  columns: 3,
  table.cell(colspan: 3, align: center)[*Epic Table*],
  align(center)[*Name*], align(center)[*Age*], align(center)[*Data*],
  table.hline(stroke: (paint: double-line, thickness: 2pt)),
  [John], [30], [None],
  [Martha], [20], [A],
  [Joseph], [35], [D]
)