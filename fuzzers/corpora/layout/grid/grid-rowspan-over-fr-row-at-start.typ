
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Fractional rows
#set page(height: 10em)
#grid(
  fill: red,
  gutter: 3pt,
  columns: 3,
  rows: (1fr, auto, 1em),
  [a], [b], grid.cell(rowspan: 3, block(height: 4em, width: 1em, fill: orange)),
  [c], [d],
  [e], [f]
)