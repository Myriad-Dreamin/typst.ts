
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  columns: 2,
  [x], [y],
  grid.header([a]),
  grid.header([b]),
  grid.cell(x: 1)[c], [d],
  grid.header([e]),
  [f], grid.cell(x: 1)[g]
)