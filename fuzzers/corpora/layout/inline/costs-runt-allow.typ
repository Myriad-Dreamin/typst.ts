
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set par(justify: true)
#set text(size: 6pt)

#let sample = [a a a a a a a a a a a a a a a a a a a a a a a a a]

#sample
#pagebreak()
#set text(costs: (runt: 0%))
#sample