
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let c = color.spot(none, red)
#box(square(size: 15pt, fill: c.tint(50%)))