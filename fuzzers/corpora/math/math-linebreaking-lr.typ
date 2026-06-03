
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// LR groups prevent linebreaking.
#let hrule(x) = box(line(length: x))
#hrule(76pt)$a+b$\
#hrule(74pt)$(a+b)$\
#hrule(74pt)$paren.l a+b paren.r$