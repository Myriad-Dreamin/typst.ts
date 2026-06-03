
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test partial border line overrides
#set page(width: auto, height: 7em, margin: (bottom: 1em))
#table(
  columns: 4,
  stroke: (x, y) => if y == 0 or y == 4 { orange } else { aqua },
  table.hline(stroke: blue, start: 1, end: 2), table.cell(stroke: red, v(3em)), table.cell(stroke: blue)[b], table.cell(stroke: green)[c], [M],
  [a], [b], [c], [M],
  [d], [e], [f], [M],
  [g], [h], [i], [M],
  table.cell(stroke: red)[a], table.cell(stroke: blue)[b], table.cell(stroke: green)[c], [M],
  table.hline(stroke: blue, start: 1, end: 2),
)