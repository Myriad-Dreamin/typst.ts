
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 5em)
#table(
  columns: 2,
  gutter: 3pt,
  stroke: red,
  inset: 5pt,
  table.cell(rowspan: 5)[a\ b\ c\ d\ e]
)

// Rowspan split without ending at the auto row