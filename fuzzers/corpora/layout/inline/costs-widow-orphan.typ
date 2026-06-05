
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 60pt)

#let sample = lorem(12)

#sample
#pagebreak()
#set text(costs: (widow: 0%, orphan: 0%))
#sample