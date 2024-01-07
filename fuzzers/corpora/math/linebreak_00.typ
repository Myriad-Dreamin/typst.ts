
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Basic breaking after binop, rel
#let hrule(x) = box(line(length: x))
#hrule(45pt)$e^(pi i)+1 = 0$\
#hrule(55pt)$e^(pi i)+1 = 0$\
#hrule(70pt)$e^(pi i)+1 = 0$
