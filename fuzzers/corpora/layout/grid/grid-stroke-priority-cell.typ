
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure cell stroke overrides always appear on top.
#table(
  columns: 2,
  stroke: black,
  table.cell(stroke: red)[a], [b],
  [c], [d],
)

#table(
  columns: 2,
  table.cell(stroke: red)[a], [b],
  [c], [d],
)