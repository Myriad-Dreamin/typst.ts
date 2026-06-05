
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let lhs = arguments(0, "1", key: "value", 3)
#let rhs = arguments(other-key: 4, key: "other value", 3)
#let result = arguments(0, "1", 3, other-key: 4, key: "other value", 3)
#test(lhs + rhs, result)
#test({lhs; rhs}, result)
#test((lhs, rhs).sum(), result)
#test((lhs, rhs).join(), result)