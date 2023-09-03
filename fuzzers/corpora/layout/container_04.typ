
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test block over multiple pages.

#set page(height: 60pt)

First!

#block[
  But, soft! what light through yonder window breaks? It is the east, and Juliet
  is the sun.
]
