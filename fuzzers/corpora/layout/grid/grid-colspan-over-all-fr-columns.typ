
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Colspan over all fractional columns shouldn't expand auto columns on finite pages
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