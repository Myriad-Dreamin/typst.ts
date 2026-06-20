
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `contains` method.
#test("abc".contains("b"), true)
#test("b" in "abc", true)
#test("1234f".contains(regex("\\d")), true)
#test(regex("\\d") in "1234f", true)
#test("abc".contains("d"), false)
#test("1234g" in "1234f", false)
#test("abc".contains(regex("^[abc]$")), false)
#test("abc".contains(regex("^[abc]+$")), true)