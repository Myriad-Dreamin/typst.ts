
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Destructuring with an empty sink and empty array.
#let (..a) = ()
#test(a, ())