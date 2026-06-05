
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  columns: 2,
  [z],
  grid.hline(stroke: red),
  grid.header(grid.cell(x: 0)[b]),
  grid.hline(stroke: 3pt),
  [w],
  [j],
  grid.header(grid.cell(x: 0, y: 9)[c]),
  [k]
)