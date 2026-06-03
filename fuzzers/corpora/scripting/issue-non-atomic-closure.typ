
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that we can't have non-atomic closures.
#let x = 1
#let c = [#(x) => (1, 2)]
#test(c.children.last(), [(1, 2)]))