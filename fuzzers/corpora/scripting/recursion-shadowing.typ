
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test redefinition.
#let f(x) = "hello"
#let f(x) = if x != none { f(none) } else { "world" }
#test(f(1), "world")