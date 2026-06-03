
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 110pt)
A
#block(width: 100%, height: 1fr, fill: aqua)[
  B #footnote[I] #footnote[II]
]
C