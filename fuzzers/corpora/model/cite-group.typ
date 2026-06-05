
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
A#[@netwok@arrgh]B \
A@netwok@arrgh B \
A@netwok @arrgh B \
A@netwok @arrgh. B \

A @netwok#[@arrgh]B \
A @netwok@arrgh, B \
A @netwok @arrgh, B \
A @netwok @arrgh. B \

A#[@netwok @arrgh @quark]B. \
A @netwok @arrgh @quark B. \
A @netwok @arrgh @quark, B.

#show bibliography: it => if target() == "html" { it }
#bibliography("/assets/bib/works.bib", style: "american-physics-society")