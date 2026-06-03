
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  columns: 2,
  fill: red,
  align: left,
  inset: 5pt,
  [ABC], [ABC],
  grid.cell(fill: blue)[C], [D],
  grid.cell(align: center)[E], [F],
  [G], grid.cell(inset: 0pt)[H]
)