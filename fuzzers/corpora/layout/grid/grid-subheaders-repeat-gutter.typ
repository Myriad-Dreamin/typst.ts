
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Gutter below the header is also repeated
#set page(height: 8em)
#grid(
  inset: (bottom: 0.5pt),
  stroke: (bottom: 1pt),
  gutter: (1pt, 6pt, 1pt),
  grid.header([a]),
  grid.header(level: 2, [b]),
  ..([c],) * 10,
)