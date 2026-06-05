
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(margin: (x: 1.1cm), columns: 2)
#set par.line(
  numbering: "1",
  number-clearance: 0.5cm,
  numbering-scope: "page"
)

A \
A \
A
#colbreak()
B \
B \
B
#pagebreak()
One \
Two \
Three
#colbreak()
Four \
Five \
Six
#page[
  Page \
  Elem
  #colbreak()
  Number \
  Reset
]
We're back
#colbreak()
Bye!