
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Cell set rules
#set table.cell(align: center)
#show table.cell: it => (it.align, it.fill, it.inset)
#set table.cell(inset: 20pt)
#table(
  align: left,
  row-gutter: 5pt,
  [A],
  table.cell(align: right)[B],
  table.cell(fill: aqua)[B],
)