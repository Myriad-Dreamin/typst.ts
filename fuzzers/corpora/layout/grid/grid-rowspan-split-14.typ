
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Second lorem must be sent to the next page, too big
#set page(width: 10cm, height: 9cm, margin: 1cm)
#set text(size: 11pt)
#table(
  columns: (1fr, 1fr, 1fr),
  align: center,
  rows: (4cm, auto),
  [A], [B], [C],
  table.cell(rowspan: 4, breakable: false, lorem(10)),
  [D],
  table.cell(rowspan: 2, breakable: false, lorem(20)),
  [E],
)