
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  columns: 1,
  grid.header([a]),
  [x],
  grid.header(level: 2, [b]),
  grid.header(level: 3, [c]),
  grid.cell(
    block(fill: red, width: 1.5em, height: 6.4em)
  ),
  [done.],
  [done.]
)