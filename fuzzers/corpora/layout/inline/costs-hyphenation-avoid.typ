
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set par(justify: true)

#let sample = [we've increased the hyphenation cost.]

#sample
#pagebreak()
#set text(costs: (hyphenation: 10000%))
#sample