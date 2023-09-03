
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test alignment with array.
#table(
  columns: (1fr, 1fr, 1fr),
  align: (left, center, right),
  [A], [B], [C]
)

// Test empty array.
#set align(center)
#table(
  columns: (1fr, 1fr, 1fr),
  align: (),
  [A], [B], [C]
)
