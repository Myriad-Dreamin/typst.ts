
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// A single linebreak at the end still counts as one line.
#let hrule(x) = box(line(length: x))
#hrule(60pt)$e^(pi i)+1 = 0\ $
