
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that block enforces consistent width across regions. Also use some
// introspection to check that measurement is working correctly.
#block(stroke: 1pt, inset: 5pt)[
  #align(right)[Hi]
  #colbreak()
  Hello @netwok
]

#show bibliography: none
#bibliography("/assets/bib/works.bib")