
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

#table(
  columns: 3,
  fill: (x, y) => (if y == 0 { aqua } else { orange }).darken(x * 15%),
  inset: (x, y) => (left: if x == 0 { 0pt } else { 5pt }, right: if x == 0 { 5pt } else { 0pt }, y: if y == 0 { 0pt } else { 5pt }),
  [A], [B], [C],
  [A], [B], [C],
)

#table(
  columns: 3,
  inset: (0pt, 5pt, 10pt),
  fill: (x, _) => aqua.darken(x * 15%),
  [A], [B], [C],
)