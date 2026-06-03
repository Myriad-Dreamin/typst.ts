
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
  [a], [b],
  grid.cell(
    block(fill: red, width: 1.5em, height: 6.4em)
  ),
  [y],
  ..([z],) * 10,
  grid.footer([f])
)