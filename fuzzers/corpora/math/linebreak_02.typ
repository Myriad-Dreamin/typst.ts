
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Multiline yet inline does not linebreak
#let hrule(x) = box(line(length: x))
#hrule(80pt)$a + b \ c + d$\
