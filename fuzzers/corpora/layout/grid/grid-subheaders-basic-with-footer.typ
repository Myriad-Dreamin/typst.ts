
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(
  grid.header([a]),
  grid.header(level: 2, [b]),
  [c],
  grid.footer([d])
)