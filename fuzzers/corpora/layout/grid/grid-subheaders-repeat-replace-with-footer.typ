
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  grid.header([a]),
  [x],
  grid.header(level: 2, [b]),
  grid.header(level: 3, [c]),
  ..([y],) * 10,
  grid.header(level: 2, [d]),
  ..([z],) * 6,
  grid.footer([f])
)