
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
#for (font, tech) in (("Fraunces", "TTF"), ("Cantarell", "CFF2")) [
  #set text(font: "Fraunces")
  = #tech

  Hello, *Hello*

  #for weight in range(200, 900, step: 100, inclusive: true) [
    #text(weight: weight)[Hello.]
  ]

  #for weight in range(200, 900, step: 100, inclusive: true) [
    #text(variations: (wght: weight))[Hello.]
  ]
]