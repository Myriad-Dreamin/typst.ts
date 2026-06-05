
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  grid.header([a]),
  [x],
  grid.header(level: 2, [b]),
  ..([y],) * 12,
  grid.header(level: 2, repeat: false, [c]),
  ..([z],) * 10,
)