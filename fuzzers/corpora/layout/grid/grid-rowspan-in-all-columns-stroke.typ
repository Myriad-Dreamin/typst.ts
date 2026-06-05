
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#table(
  columns: 2,
  table.cell(stroke: (bottom: red))[a], [b],
  table.hline(stroke: green),
  table.cell(stroke: (top: yellow, left: green, right: aqua, bottom: blue), colspan: 1, rowspan: 2)[d], table.cell(colspan: 1, rowspan: 2)[e],
  [f],
  [g]
)