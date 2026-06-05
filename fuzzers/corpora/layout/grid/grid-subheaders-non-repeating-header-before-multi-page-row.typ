
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 6em)
#grid(
  grid.header(repeat: false, [h]),
  [row #colbreak() row]
)