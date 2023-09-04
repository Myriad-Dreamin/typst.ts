
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 250pt, width: 150pt)

= Introduction
#lorem(10) #footnote[Lots of Latin]

#figure(
  placement: bottom,
  caption: [A glacier #footnote[Lots of Ice]],
  image("/assets/files/glacier.jpg", width: 80%),
)

#lorem(40)

#figure(
  placement: top,
  caption: [An important],
  image("/assets/files/diagram.svg", width: 80%),
)
