
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test set rules on cells and folding
#set table.cell(stroke: 4pt)
#set table.cell(stroke: blue)
#set table.hline(stroke: red)
#set table.hline(stroke: 0.75pt)
#set table.vline(stroke: 0.75pt)
#set table.vline(stroke: aqua)

#table(
  columns: 3,
  gutter: 3pt,
  inset: 5pt,
  [a], [b], table.vline(position: end), [c],
  [d], [e], [f],
  table.hline(position: bottom),
  [g], [h], [i],
)