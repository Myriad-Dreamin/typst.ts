
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Cell show rule
#show table.cell: it => [Zz]

#table(
  align: left,
  fill: red,
  stroke: blue,
  columns: 2,
  [AAAAA], [BBBBB],
  [A], [B],
  table.cell(align: right)[C], [D],
  align(right)[E], [F],
  align(horizon)[G], [A\ A\ A]
)