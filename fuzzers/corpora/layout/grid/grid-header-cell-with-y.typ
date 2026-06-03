
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  grid.cell(y: 1)[a],
  grid.header(grid.cell(y: 0)[b]),
  grid.cell(y: 2)[c]
)