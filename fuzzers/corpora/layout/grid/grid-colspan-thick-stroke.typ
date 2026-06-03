
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 300pt)
#table(
  columns: (2em, 2em, auto, auto),
  stroke: 5pt,
  [A], [B], [C], [D],
  table.cell(colspan: 4, lorem(20)),
  [A], table.cell(colspan: 2)[BCBCBCBC], [D]
)