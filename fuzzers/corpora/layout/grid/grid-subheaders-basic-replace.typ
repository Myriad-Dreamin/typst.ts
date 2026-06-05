
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  grid.header([a]),
  [x],
  grid.header(level: 2, [b]),
  [y],
  grid.header(level: 2, [c]),
  [z],
)