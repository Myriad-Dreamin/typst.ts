
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  columns: 2,
  stroke: black,
  inset: 5pt,
  grid.header(grid.cell(x: 0)[b1], grid.cell(x: 0)[b2]),
  // This should skip the header
  grid.cell(x: 1)[c]
)