
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 9em)
#grid(
  columns: 2,
  column-gutter: 4pt,
  row-gutter: (0pt, 4pt, 8pt, 4pt),
  inset: (bottom: 0.5pt),
  stroke: (bottom: 1pt),
  grid.header([a]),
  [x],
  grid.header(level: 2, [b]),
  [y],
  grid.header(level: 3, [c]),
  [z], [z],
  grid.cell(
    rowspan: 5,
    block(fill: red, width: 1.5em, height: 6.4em)
  ),
  [cell],
  [cell],
  [a\ b],
  grid.cell(x: 0)[end],
)