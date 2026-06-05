
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test left and right for grid vlines.
#grid(
  columns: 3,
  inset: 5pt,
  grid.vline(stroke: green, position: left), grid.vline(stroke: red, position: right), [a],
  grid.vline(stroke: 2pt, position: left), grid.vline(stroke: red, position: right), [b],
  grid.vline(stroke: 2pt, position: left), grid.vline(stroke: red, position: right), [c],
  grid.vline(stroke: 2pt, position: left)
)

#grid(
  columns: 3,
  inset: 5pt,
  gutter: 3pt,
  grid.vline(stroke: green, position: left), grid.vline(stroke: red, position: right), [a],
  grid.vline(stroke: blue, position: left), grid.vline(stroke: red, position: right), [b],
  grid.vline(stroke: blue, position: left), grid.vline(stroke: red, position: right), [c],
  grid.vline(stroke: 2pt, position: left)
)