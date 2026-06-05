
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  [b],
  grid.header(
    [a\ ] * 5,
    repeat: true,
  ),
  [c],
)