
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test destructuring assignments.

#let a = none
#let b = none
#let c = none
#((a,) = (1,))
#test(a, 1)

#((_, a, b, _) = (1, 2, 3, 4))
#test(a, 2)
#test(b, 3)

#((a, b, ..c) = (1, 2, 3, 4, 5, 6))
#test(a, 1)
#test(b, 2)
#test(c, (3, 4, 5, 6))

#((a: a, b, x: c) = (a: 1, b: 2, x: 3))
#test(a, 1)
#test(b, 2)
#test(c, 3)

#let a = (1, 2)
#((a: a.at(0), b) = (a: 3, b: 4))
#test(a, (3, 2))
#test(b, 4)

#let a = (1, 2)
#((a.at(0), b) = (3, 4))
#test(a, (3, 2))
#test(b, 4)

#((a, ..b) = (1, 2, 3, 4))
#test(a, 1)
#test(b, (2, 3, 4))

#let a = (1, 2)
#((b, ..a.at(0)) = (1, 2, 3, 4))
#test(a, ((2, 3, 4), 2))
#test(b, 1)