
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test spot color with name set to none.
#let varnish = color.spot(none, luma(0%))
#let layer = varnish.tint(100%)
#box(square(size: 20pt, fill: layer))