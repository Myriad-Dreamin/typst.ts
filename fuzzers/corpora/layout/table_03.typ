
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test inset.
#table(
  columns: 3,
  inset: 10pt,
  [A], [B], [C]
)

#table(
  columns: 3,
  inset: (y: 10pt),
  [A], [B], [C]
)

#table(
  columns: 3,
  inset: (left: 20pt, rest: 10pt),
  [A], [B], [C]
)

#table(
  columns: 2,
  inset: (
    left: 20pt,
    right: 5pt,
    top: 10pt,
    bottom: 3pt,
  ),
  [A],
  [B],
)
