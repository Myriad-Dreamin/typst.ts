
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Positioning cells in a different order than they appear
#grid(
  columns: 2,
  [A], [B],
  grid.cell(x: 1, y: 2)[C], grid.cell(x: 0, y: 2)[D],
  grid.cell(x: 1, y: 1)[E], grid.cell(x: 0, y: 1)[F],
)