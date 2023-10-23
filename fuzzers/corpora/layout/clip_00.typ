
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test box clipping with a rectangle
Hello #box(width: 1em, height: 1em, clip: false)[#rect(width: 3em, height: 3em, fill: red)]
world 1

Space

Hello #box(width: 1em, height: 1em, clip: true)[#rect(width: 3em, height: 3em, fill: red)]
world 2
