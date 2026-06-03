
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test comma placement in destructuring assignment.
#let array = (1, 2, 3)
#((key: array.at(1)) = (key: "hi"))
#test(array, (1, "hi", 3))

#let array = (1, 2, 3)
#((array.at(1)) = ("hi"))
#test(array, (1, "hi", 3))

#let array = (1, 2, 3)
#((array.at(1),) = ("hi",))
#test(array, (1, "hi", 3))

#let array = (1, 2, 3)
#((array.at(1)) = ("hi",))
#test(array, (1, ("hi",), 3))