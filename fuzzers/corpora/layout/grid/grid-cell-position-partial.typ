
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Partial positioning
#grid(
  columns: 3,
  rows: 1.5em,
  inset: 5pt,
  fill: aqua,
  [A], grid.cell(y: 1, fill: green)[B], [C], grid.cell(x: auto, y: 1, fill: green)[D], [E],
  grid.cell(y: 2, fill: green)[F], grid.cell(x: 0, fill: orange)[G], grid.cell(x: 0, y: auto, fill: orange)[H],
  grid.cell(x: 1, fill: orange)[I]
)

#table(
  columns: 3,
  rows: 1.5em,
  inset: 5pt,
  fill: aqua,
  [A], table.cell(y: 1, fill: green)[B], [C], table.cell(x: auto, y: 1, fill: green)[D], [E],
  table.cell(y: 2, fill: green)[F], table.cell(x: 0, fill: orange)[G], table.cell(x: 0, y: auto, fill: orange)[H],
  table.cell(x: 1, fill: orange)[I]
)