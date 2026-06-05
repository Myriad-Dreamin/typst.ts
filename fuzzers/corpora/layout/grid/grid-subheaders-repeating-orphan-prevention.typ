
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#v(4.5em)
#grid(
  grid.header(repeat: true, level: 2, [L2]),
  grid.header(repeat: true, level: 4, [L4]),
  [a]
)