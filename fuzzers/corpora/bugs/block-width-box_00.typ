
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#block(width: 100%, fill: red, box("a box"))

#block(width: 100%, fill: red, [#box("a box") #box()])
