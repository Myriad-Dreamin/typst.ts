
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Table should be just one row. [c] appears at the third column.
#set page(margin: 2pt)
#set text(6pt)
#table(
  columns: 3,
  inset: 1.5pt,
  table.footer(
    table.cell(y: 0)[a],
    table.hline(stroke: red),
    table.hline(y: 1, stroke: aqua),
    table.cell(y: 0)[b],
    [c]
  )
)