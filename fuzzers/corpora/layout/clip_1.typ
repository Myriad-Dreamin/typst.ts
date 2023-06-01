// Test clipping with the `box` and `block` containers.

#set page(width: 120pt, height: auto, margin: 10pt)

// Test box clipping with a rectangle
Hello #box(width: 1em, height: 1em, clip: false)[#rect(width: 3em, height: 3em, fill: red)]
world 1

Space

Hello #box(width: 1em, height: 1em, clip: true)[#rect(width: 3em, height: 3em, fill: red)] 
world 2
