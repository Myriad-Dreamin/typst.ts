
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  grid.header([a]),
  [x],
  grid.header(level: 2, [b]),
  grid.header(level: 3, [c]),
  ..([y],) * 9,
  grid.header(level: 2, repeat: false, [d]),
  ..([z],) * 6,
)