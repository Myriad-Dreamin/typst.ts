
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure grids expand enough for the given rows.
#grid(
  columns: (2em, 2em),
  rows: (2em,) * 4,
  fill: red,
  stroke: aqua,
  [a]
)