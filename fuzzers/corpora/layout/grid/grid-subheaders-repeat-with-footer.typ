
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  grid.header([a]),
  [m],
  grid.header(level: 2, [b]),
  ..([c],) * 10,
  grid.footer([f])
)