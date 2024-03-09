
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Inline, in a box, doesn't linebreak.
#let hrule(x) = box(line(length: x))
#hrule(80pt)#box($a+b$)
