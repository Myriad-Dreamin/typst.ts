
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Here, `a` is not duplicate, where it was previously identified as one.
#let f((a: b), (c,), a) = (a, b, c)
#test(f((a: 1), (2,), 3), (3, 1, 2))