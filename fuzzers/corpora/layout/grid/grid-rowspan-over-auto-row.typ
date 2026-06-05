
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Auto row expansion
#set page(height: 10em)
#grid(
  columns: (1em, 1em),
  rows: (0.5em, 0.5em, auto),
  fill: orange,
  gutter: 3pt,
  grid.cell(rowspan: 4, [x x x x] + place(bottom)[*Bot*]),
  [a],
  [b],
  [c],
  [d]
)