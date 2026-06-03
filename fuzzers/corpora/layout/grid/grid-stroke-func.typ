
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  columns: 3,
  inset: 3pt,
  stroke: (x, _) => (right: (5pt, (dash: "dotted")).at(calc.rem(x, 2)), bottom: (dash: "densely-dotted")),
  grid.vline(x: 0, stroke: red),
  grid.vline(x: 1, stroke: red),
  grid.vline(x: 2, stroke: red),
  grid.vline(x: 3, stroke: red),
  grid.hline(y: 0, end: 1, stroke: blue),
  grid.hline(y: 1, end: 1, stroke: blue),
  grid.cell[a],
  [b], [c]
)