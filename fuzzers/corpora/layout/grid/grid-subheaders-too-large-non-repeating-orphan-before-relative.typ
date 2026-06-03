
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  rows: (auto, auto, auto, 3em),
  grid.header([1]),
  grid.header([a\ ] * 2, level: 2, repeat: false),
  grid.header([2], level: 3),
  rect(width: 10pt, height: 3em, fill: red),
)