
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  grid.footer(grid.cell(y: 2)[b]),
  grid.cell(y: 0)[a],
  grid.cell(y: 1)[c],
)