
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set text(8pt)
#outline()
#set text(2pt)
#hide(block(grid(
  [= A],
  [= B],
  block(grid(
    [= C],
    [= D],
  ))
)))
