
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Destructuring with an empty sink and empty dict.
#let (..a) = (:)
#test(a, (:))