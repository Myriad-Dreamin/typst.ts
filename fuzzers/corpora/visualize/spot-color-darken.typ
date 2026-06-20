
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test darken on spot colors.
#let pantone = color.spot("PANTONE 185 C", rgb(89.4%, 0.7%, 17%))
#let base = pantone.tint(50%)
#let dark = base.darken(25%)
#box(square(size: 15pt, fill: base))
#box(square(size: 15pt, fill: dark))