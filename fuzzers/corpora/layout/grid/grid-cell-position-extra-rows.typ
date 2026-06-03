
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Creating more rows by positioning out of bounds
#grid(
  columns: 3,
  rows: 1.5em,
  inset: 5pt,
  fill: (x, y) => if (x, y) == (0, 0) { blue } else if (x, y) == (2, 3) { red } else { green },
  [A],
  grid.cell(x: 2, y: 3)[B]
)

#table(
  columns: (3em, 1em, 3em),
  rows: 1.5em,
  inset: (top: 0pt, bottom: 0pt, rest: 5pt),
  fill: (x, y) => if (x, y) == (0, 0) { blue } else if (x, y) == (2, 3) { red } else { green },
  align: (x, y) => (left, center, right).at(x),
  [A],
  table.cell(x: 2, y: 3)[B]
)