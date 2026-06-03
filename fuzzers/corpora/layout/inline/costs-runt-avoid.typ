
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set par(justify: true)

#let sample = [please avoid runts in this text.]

#sample
#pagebreak()
#set text(costs: (runt: 10000%))
#sample