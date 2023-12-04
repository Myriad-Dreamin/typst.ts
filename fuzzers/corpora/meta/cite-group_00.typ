
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

#set text(0pt)
#bibliography("/assets/files/works.bib")
