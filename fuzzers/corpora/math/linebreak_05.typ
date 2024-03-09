
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// A relation followed by a relation doesn't linebreak
#let hrule(x) = box(line(length: x))
#hrule(70pt)$a < = b$\
#hrule(74pt)$a < = b$
