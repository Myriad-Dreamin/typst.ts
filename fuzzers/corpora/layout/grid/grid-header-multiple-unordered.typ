
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 4em)
#grid(
  grid.header(grid.cell(x: 0, y: 4)[y]),
  grid.header([x]),
  [a],
  [b],
  [c],
  [d],
  [e],
  [f],
)