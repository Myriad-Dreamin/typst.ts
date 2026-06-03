
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  gutter: 3pt,
  grid.header([a]),
  [x],
  grid.header(level: 2, [b]),
  ..([y],) * 9,
  grid.header(level: 2, [c]),
  [z \ z],
  ..([z],) * 3,
)