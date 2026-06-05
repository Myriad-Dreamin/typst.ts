
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  grid.header([a]),
  grid.header(level: 2, [b]),
  ..([c],) * 10,
)