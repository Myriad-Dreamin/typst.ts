
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let c = color.spot("all", blue)
#box(square(size: 15pt, fill: c.tint(75%)))