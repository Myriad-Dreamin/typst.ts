
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `slice` method.
#test("abc".slice(1, 2), "b")
#test("abc🏡def".slice(2, 7), "c🏡")
#test("abc🏡def".slice(2, -2), "c🏡d")
#test("abc🏡def".slice(-3, -1), "de")
#test("x🏡yz".slice(-2, count: 2), "yz")
#test("x🏡yz".slice(-7, count: 7), "x🏡yz")