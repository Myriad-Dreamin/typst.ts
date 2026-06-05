
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  columns: 1,
  [a],
  grid.hline(stroke: red),
  grid.footer([b]),
  grid.hline(stroke: 3pt),
)