
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Using position: bottom and position: end without gutter should be the same
// as placing a line after the next track.
#table(
  columns: 3,
  stroke: blue,
  table.hline(end: 2, stroke: red),
  table.hline(end: 2, stroke: aqua, position: bottom),
  table.vline(end: 2, stroke: green), [a], table.vline(end: 2, stroke: green), [b], table.vline(end: 2, stroke: aqua, position: end), table.vline(end: 2, stroke: green), [c], table.vline(end: 2, stroke: green),
  table.hline(end: 2, stroke: 5pt),
  [d], [e], [f],
  table.hline(end: 2, stroke: red),
  [g], [h], [i],
  table.hline(end: 2, stroke: red),
)