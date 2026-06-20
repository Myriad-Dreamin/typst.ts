
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(background: pad(10pt, rect(
  width: 100%,
  height: 100%,
  stroke: 0.5pt + blue
)))
#set par(justify: true)

#let space = h(100% - 3.1em)

#space;Foo Bar Buzz

#space;Foo Bar#sym.zws;Buzz