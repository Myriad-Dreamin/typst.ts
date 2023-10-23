
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test cliping text
#block(width: 5em, height: 2em, clip: false, stroke: 1pt + black)[
  But, soft! what light through
]

#v(2em)

#block(width: 5em, height: 2em, clip: true, stroke: 1pt + black)[
  But, soft! what light through yonder window breaks? It is the east, and Juliet
  is the sun.
]
