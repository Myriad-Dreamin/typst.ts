
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// On infinite pages, colspan over all fractional columns SHOULD expand auto columns
#set page(width: auto)
#table(
  columns: (1fr, 1fr, auto),
  [A], [B], [C],
  [D], [E], [F]
)
#table(
  columns: (1fr, 1fr, auto),
  table.cell(colspan: 3, lorem(8)),
  [A], [B], [C],
  [D], [E], [F]
)