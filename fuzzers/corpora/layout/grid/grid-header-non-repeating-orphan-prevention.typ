
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 5em)
#v(2em)
#grid(
  grid.header(repeat: false)[*Abc*],
  [a],
  [b],
  [c],
  [d]
)