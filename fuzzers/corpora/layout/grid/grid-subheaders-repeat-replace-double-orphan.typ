
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  grid.header([a]),
  [x],
  grid.header(level: 2, [b]),
  ..([y],) * 11,
  grid.header(level: 2, [c]),
  grid.header(level: 3, [d]),
  ..([z],) * 10,
)