
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// This is a fun one.
#set par(justify: true)
#show regex("\\S"): letter => box(stroke: 1pt, inset: 2pt, upper(letter))
#lorem(5)