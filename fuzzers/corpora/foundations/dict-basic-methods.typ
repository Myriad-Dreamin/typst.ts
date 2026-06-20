
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test dictionary methods.
#let dict = (a: 3, c: 2, b: 1)
#test("c" in dict, true)
#test(dict.len(), 3)
#test(dict.values(), (3, 2, 1))
#test(dict.pairs().map(p => p.first() + str(p.last())).join(), "a3c2b1")

#test(dict.remove("c"), 2)
#test("c" in dict, false)
#test(dict, (a: 3, b: 1))