// Test clipping with the `box` and `block` containers.

#set page(width: 120pt, height: 60pt, margin: 10pt)

// Test block clipping over multiple pages.

First!

#block(height: 4em, clip: true, stroke: 1pt + black)[
  But, soft! what light through yonder window breaks? It is the east, and Juliet
  is the sun.
]
