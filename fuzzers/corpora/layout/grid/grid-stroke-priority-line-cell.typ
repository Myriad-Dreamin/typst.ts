
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Hlines and vlines should always appear on top of cell strokes.
#table(
  columns: 3,
  stroke: aqua,
  table.vline(stroke: red, position: end), [a], table.vline(stroke: red), [b], [c],
  table.cell(stroke: blue)[d], [e], [f],
  table.hline(stroke: red),
  [g], table.cell(stroke: blue)[h], [i],
)

#table(
  columns: 3,
  gutter: 3pt,
  stroke: aqua,
  table.vline(stroke: red, position: end), [a], table.vline(stroke: red), [b], [c],
  table.cell(stroke: blue)[d], [e], [f],
  table.hline(stroke: red),
  [g], table.cell(stroke: blue)[h], [i],
)