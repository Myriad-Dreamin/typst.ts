
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 5em)
#grid(
  columns: 2,
  stroke: red,
  inset: 5pt,
  grid.cell(rowspan: 5)[a\ b\ c\ d\ e]
)