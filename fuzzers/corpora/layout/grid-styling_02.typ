
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test general alignment.
#grid(
  columns: 3,
  align: left,
  [Hello], [Hello], [Hello],
  [A], [B], [C],
)

// Test alignment with a function.
#grid(
  columns: 3,
  align: (x, y) => (left, center, right).at(x),
  [Hello], [Hello], [Hello],
  [A], [B], [C],
)

// Test alignment with array.
#grid(
  columns: (1fr, 1fr, 1fr),
  align: (left, center, right),
  [A], [B], [C]
)

// Test empty array.
#set align(center)
#grid(
  columns: (1fr, 1fr, 1fr),
  align: (),
  [A], [B], [C]
)

a
