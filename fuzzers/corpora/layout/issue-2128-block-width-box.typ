
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test box in 100% width block.
#block(width: 100%, fill: red, box("a box"))
#block(width: 100%, fill: red, [#box("a box") #box()])