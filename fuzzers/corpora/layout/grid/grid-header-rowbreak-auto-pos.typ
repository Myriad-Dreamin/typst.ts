
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  columns: 2,
  [x],
  grid.hline(stroke: red),
  grid.header([a]),
  grid.hline(stroke: 3pt),
  [y],
  grid.header(),
  [z],
)