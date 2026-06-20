
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test different tint levels of the same spot colorant.
#let pantone = color.spot("PANTONE 185 C", rgb(89.4%, 0.7%, 17%))
#box(square(size: 15pt, fill: pantone.tint(100%)))
#box(square(size: 15pt, fill: pantone.tint(75%)))
#box(square(size: 15pt, fill: pantone.tint(50%)))
#box(square(size: 15pt, fill: pantone.tint(25%)))
#box(square(size: 15pt, fill: pantone.tint(0%)))