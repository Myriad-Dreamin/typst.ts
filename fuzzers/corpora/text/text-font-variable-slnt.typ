
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
#set text(font: "Roboto Flex")

Hello, _Hello_

#text(style: "italic")[Hello],
#text(style: "oblique")[Hello]

#for slnt in range(0, -10, step: -2, inclusive: true) [
  #text(variations: (slnt: slnt))[Hello.]
]