
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test basic spot color creation and rendering.
#let pantone = color.spot("PANTONE 2221 C", eastern)
#let tinted = pantone.tint(80%)
#box(square(size: 20pt, fill: tinted))