
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show grid.cell: it => (it.align, it.fill)
#grid(
  align: left,
  row-gutter: 5pt,
  [A],
  grid.cell(align: right)[B],
  grid.cell(fill: aqua)[B],
)