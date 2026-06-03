
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Destructuring with multiple placeholders.
#let (a, _, c, _) = (1, 2, 3, 4)
#test(a, 1)
#test(c, 3)