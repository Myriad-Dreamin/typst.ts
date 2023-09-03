
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test block clipping over multiple pages.

#set page(height: 60pt)

First!

#block(height: 4em, clip: true, stroke: 1pt + black)[
  But, soft! what light through yonder window breaks? It is the east, and Juliet
  is the sun.
]
