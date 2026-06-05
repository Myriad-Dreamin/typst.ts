
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  grid.header(repeat: false, [a]),
  [x],
  grid.header(level: 2, repeat: false, [b]),
  ..([y],) * 10,
)