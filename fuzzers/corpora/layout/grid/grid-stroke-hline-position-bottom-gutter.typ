
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Position: bottom and position: end with gutter should have a visible effect
// of moving the lines after the next track.
#table(
  columns: 3,
  gutter: 3pt,
  stroke: blue,
  table.hline(end: 2, stroke: red),
  table.hline(end: 2, stroke: aqua, position: bottom),
  table.vline(end: 2, stroke: green), [a], table.vline(end: 2, stroke: green), table.vline(end: 2, position: end, stroke: orange), [b], table.vline(end: 2, stroke: aqua, position: end), table.vline(end: 2, stroke: green), [c], table.vline(end: 2, stroke: green),
  [d], [e], [f],
  table.hline(end: 2, stroke: red),
  [g], [h], [ie],
  table.hline(end: 2, stroke: green),
)