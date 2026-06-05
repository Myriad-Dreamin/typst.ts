
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test left and right for table vlines.
#table(
  columns: 3,
  inset: 5pt,
  table.vline(stroke: green, position: left), table.vline(stroke: red, position: right), [a],
  table.vline(stroke: 2pt, position: left), table.vline(stroke: red, position: right), [b],
  table.vline(stroke: 2pt, position: left), table.vline(stroke: red, position: right), [c],
  table.vline(stroke: 2pt, position: left)
)

#table(
  columns: 3,
  inset: 5pt,
  gutter: 3pt,
  table.vline(stroke: green, position: left), table.vline(stroke: red, position: right), [a],
  table.vline(stroke: blue, position: left), table.vline(stroke: red, position: right), [b],
  table.vline(stroke: blue, position: left), table.vline(stroke: red, position: right), [c],
  table.vline(stroke: 2pt, position: left)
)