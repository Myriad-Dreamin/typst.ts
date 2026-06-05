
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test multiple regions
#set page(height: 5em)
#grid(
  stroke: red,
  fill: aqua,
  columns: 4,
  [a], [b], [c], [d],
  [a], grid.cell(colspan: 2)[e, f, g, h, i], [f],
  [e], [g], grid.cell(colspan: 2)[eee\ e\ e\ e],
  grid.cell(colspan: 4)[eeee e e e]
)