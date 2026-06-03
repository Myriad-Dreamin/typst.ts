
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that we can't have non-atomic destructuring.
#let x = 1
#let c = [#() = ()]
#test(c.children.last(), [()])