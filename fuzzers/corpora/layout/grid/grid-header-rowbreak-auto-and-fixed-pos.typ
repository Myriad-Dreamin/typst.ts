
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  columns: 2,
  [a],
  grid.header([x]),
  [b],
  grid.header(grid.cell(x: 0, y: 3)[y]),
  [c]
)