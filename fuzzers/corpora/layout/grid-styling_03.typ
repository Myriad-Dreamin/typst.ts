
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test inset.
#grid(
  columns: (1fr,) * 3,
  stroke: 2pt + rgb("333"),
  inset: 5pt,
  [A], [B], [C], [], [], [D \ E \ F \ \ \ G], [H],
)

#grid(
  columns: 3,
  inset: 10pt,
  fill: blue,
  [A], [B], [C]
)

#grid(
  columns: 3,
  inset: (y: 10pt),
  [A], [B], [C]
)

#grid(
  columns: 3,
  inset: (left: 20pt, rest: 10pt),
  stroke: 3pt + red,
  [A], [B], [C]
)

#grid(
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
