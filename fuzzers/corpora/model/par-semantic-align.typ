
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show par: highlight
#show bibliography: none
#set block(width: 100%, stroke: 1pt, inset: 5pt)

#bibliography("/assets/bib/works.bib")

#block[
  #set align(right)
  Hello
]

#block[
  #set align(right)
  Hello
  @netwok
]

#block[
  Hello
  #align(right)[World]
  You
]

#block[
  Hello
  #align(right)[@netwok]
  You
]