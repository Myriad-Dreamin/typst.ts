
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  columns: 2,
  [a], [],
  [b], [],
  stroke: red,
  inset: 5pt,
  grid.cell(x: 1, y: 3, rowspan: 4)[b],
  grid.cell(y: 2, rowspan: 2)[a],
  grid.footer(),
  grid.cell(y: 4)[d],
  grid.cell(y: 5)[e],
  grid.cell(y: 6)[f],
)