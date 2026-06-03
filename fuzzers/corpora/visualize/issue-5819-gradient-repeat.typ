
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure the gradient constructor generates monotonic stops which can be fed
// back into the gradient constructor itself.
#let my-gradient = gradient.linear(red, blue).repeat(5)
#let _ = gradient.linear(..my-gradient.stops())
#let my-gradient2 = gradient.linear(red, blue).repeat(5, mirror: true)
#let _ = gradient.linear(..my-gradient2.stops())