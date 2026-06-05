
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em)
#grid(
  grid.header(
    [a\ ] * 5,
    repeat: true,
  ),
  [b],
  grid.footer(
    [c],
    repeat: true,
  )
)