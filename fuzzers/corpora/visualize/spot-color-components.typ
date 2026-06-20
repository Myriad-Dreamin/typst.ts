
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test spot color components method.
#let pantone = color.spot("PANTONE 2221 C", eastern)
#let tinted = pantone.tint(80%)
#test(tinted.components(), (80%,))
#test(tinted.components(alpha: false), (80%,))