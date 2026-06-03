
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  columns: 2,
  [a],
  grid.header(grid.cell(x: 0, y: 2)[y]),
  [b],
  grid.header([x]),
  [c]
)