
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
#table(
  columns: 2,
  [A], [B],
  [C], [D],
  table.vline(end: 3),
  table.hline(end: 3),
)