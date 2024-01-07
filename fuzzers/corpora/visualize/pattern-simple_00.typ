
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: auto, height: auto, margin: 0pt)
#let pat = pattern(size: (10pt, 10pt), line(stroke: 4pt, start: (0%, 0%), end: (100%, 100%)))
#rect(width: 50pt, height: 50pt, fill: pat)
