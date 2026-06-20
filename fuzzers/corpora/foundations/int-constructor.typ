
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test conversion to numbers.
#test(int(false), 0)
#test(int(true), 1)
#test(int(10), 10)
#test(int("0"), 0)
#test(int("+150"), 150)
#test(int("-834"), -834)
#test(int("beef", base: 16), 48879)
#test(int("-cAfFe", base: 16), -831486)
#test(int("10", base: 2), 2)
#test(int("644", base: 8), 420)
#test(int("\u{2212}79"), -79)
#test(int("9223372036854775807"), int.max)
#test(int("-9223372036854775808"), int.min)
#test(int("7FFFFFFFFFFFFFFF", base: 16), int.max)
#test(int("-8000000000000000", base: 16), int.min)
#test(int(10 / 3), 3)
#test(int(-58.34), -58)
#test(int(decimal("92492.193848921")), 92492)
#test(int(decimal("-224.342211")), -224)