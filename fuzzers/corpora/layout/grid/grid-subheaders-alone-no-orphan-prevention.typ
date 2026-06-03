
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 5.3em)
#v(2em)
#grid(
  grid.header([L1]),
  grid.header(level: 2, [L2]),
)