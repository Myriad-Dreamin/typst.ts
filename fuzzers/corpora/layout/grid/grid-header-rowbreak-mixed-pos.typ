
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  columns: 2,
  [a],
  grid.header([x], grid.cell(x: 0)[b]),
  [c],
  grid.hline(stroke: red),
  grid.header([y], grid.cell(x: 0, y: 8)[d]),
  grid.hline(stroke: 3pt),
  [e]
)