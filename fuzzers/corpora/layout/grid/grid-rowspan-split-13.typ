
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Inset moving to next region bug
#set page(width: 10cm, height: 2.5cm, margin: 0.5cm)
#set text(size: 11pt)
#table(
  columns: (1fr, 1fr, 1fr),
  [A],
  [B],
  [C],
  [D],
  table.cell(rowspan: 2, lorem(4)),
  [E],
  [F],
  [G],
)