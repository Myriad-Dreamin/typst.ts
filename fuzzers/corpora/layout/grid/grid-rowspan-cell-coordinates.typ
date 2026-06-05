
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Cell coordinate tests
#set page(height: 10em)
#show table.cell: it => [(#it.x, #it.y)]
#table(
  columns: 3,
  fill: red,
  [a], [b], table.cell(rowspan: 2)[c],
  table.cell(colspan: 2)[d],
  table.cell(colspan: 3, rowspan: 10)[a],
  table.cell(colspan: 2)[b],
)
#table(
  columns: 3,
  gutter: 3pt,
  fill: red,
  [a], [b], table.cell(rowspan: 2)[c],
  table.cell(colspan: 2)[d],
  table.cell(colspan: 3, rowspan: 9)[a],
  table.cell(colspan: 2)[b],
)