
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 70pt)
#block[This file tests a bug where an almost empty page occurs.]
#block[
  The text in this second block was torn apart and split up for
  some reason beyond my knowledge.
]
