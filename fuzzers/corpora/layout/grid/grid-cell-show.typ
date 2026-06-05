
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Cell show rule
#show grid.cell: it => [Zz]

#grid(
  align: left,
  fill: red,
  stroke: blue,
  inset: 5pt,
  columns: 2,
  [AAAAA], [BBBBB],
  [A], [B],
  grid.cell(align: right)[C], [D],
  align(right)[E], [F],
  align(horizon)[G], [A\ A\ A]
)