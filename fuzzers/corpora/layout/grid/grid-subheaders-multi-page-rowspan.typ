
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  columns: 2,
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
  [cell]
)