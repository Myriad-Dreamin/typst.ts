
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
#set text(font: "Roboto Flex")

Hello

#for stretch in range(50, 150, step: 10) [
  #text(stretch: stretch * 1%)[Hello.]
]

#for stretch in range(50, 150, step: 10) [
  #text(variations: (wdth: stretch))[Hello.]
]