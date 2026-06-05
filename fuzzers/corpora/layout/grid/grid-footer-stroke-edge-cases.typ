
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test footer stroke priority edge case
#set page(height: 10em)
#table(
  columns: 2,
  stroke: black,
  ..(table.cell(stroke: aqua)[d],) * 8,
  table.footer(
    table.cell(rowspan: 2, colspan: 2)[a],
    [c], [d]
  )
)