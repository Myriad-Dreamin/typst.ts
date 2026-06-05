
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Colspan over only some fractional columns will not trigger the heuristic, and
// the auto column will expand more than it should. The table looks off, as a result.
#table(
  columns: (1fr, 1fr, auto),
  [], table.cell(colspan: 2, lorem(8)),
  [A], [B], [C],
  [D], [E], [F]
)