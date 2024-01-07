
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Page breaks can happen after a relation even if there is no 
// explicit space.
#let hrule(x) = box(line(length: x))
#hrule(90pt)$<;$\
#hrule(95pt)$<;$\
#hrule(90pt)$<)$\
#hrule(95pt)$<)$
