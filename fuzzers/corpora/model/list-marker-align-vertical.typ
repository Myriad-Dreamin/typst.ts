
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
- #box(fill: teal, inset: 10pt)[a]

#set list(marker-align: top)
- #box(fill: teal, inset: 10pt)[b]

#set list(marker-align: horizon)
- #box(fill: teal, inset: 10pt)[c]

#set list(marker-align: bottom)
- #box(fill: teal, inset: 10pt)[d]