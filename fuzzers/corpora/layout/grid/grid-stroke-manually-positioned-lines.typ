
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 5em)
#table(
  columns: 3,
  inset: 3pt,
  table.hline(y: 0, end: none, stroke: 3pt + blue),
  table.vline(x: 0, end: none, stroke: 3pt + green),
  table.hline(y: 5, end: none, stroke: 3pt + red),
  table.vline(x: 3, end: none, stroke: 3pt + yellow),
  [a], [b], [c],
  [a], [b], [c],
  [a], [b], [c],
  [a], [b], [c],
  [a], [b], [c],
)