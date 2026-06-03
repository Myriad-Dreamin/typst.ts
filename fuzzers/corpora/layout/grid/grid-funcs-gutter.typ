
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test interaction with gutters.
#grid(
  columns: (3em, 3em),
  fill: (x, y) => (red, blue).at(calc.rem(x, 2)),
  align: (x, y) => (left, right).at(calc.rem(y, 2)),
  [A], [B],
  [C], [D],
  [E], [F],
  [G], [H]
)

#grid(
  columns: (3em, 3em),
  fill: (x, y) => (red, blue).at(calc.rem(x, 2)),
  align: (x, y) => (left, right).at(calc.rem(y, 2)),
  row-gutter: 5pt,
  [A], [B],
  [C], [D],
  [E], [F],
  [G], [H]
)

#grid(
  columns: (3em, 3em),
  fill: (x, y) => (red, blue).at(calc.rem(x, 2)),
  align: (x, y) => (left, right).at(calc.rem(y, 2)),
  column-gutter: 5pt,
  [A], [B],
  [C], [D],
  [E], [F],
  [G], [H]
)

#grid(
  columns: (3em, 3em),
  fill: (x, y) => (red, blue).at(calc.rem(x, 2)),
  align: (x, y) => (left, right).at(calc.rem(y, 2)),
  gutter: 5pt,
  [A], [B],
  [C], [D],
  [E], [F],
  [G], [H]
)