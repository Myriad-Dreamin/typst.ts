
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Fractional rows
// They cause the auto row to expand more than needed.
#set page(height: 10em)
#grid(
  fill: red,
  gutter: 3pt,
  columns: 3,
  rows: (1em, auto, 1fr),
  [a], [b], grid.cell(rowspan: 3, block(height: 4em, width: 1em, fill: orange)),
  [c], [d],
  [e], [f]
)