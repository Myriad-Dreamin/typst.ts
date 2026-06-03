
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  grid.header([1]),
  grid.header([a\ ] * 2, level: 2, repeat: false),
  grid.header([2], level: 3),
  [b\ b\ b],
)