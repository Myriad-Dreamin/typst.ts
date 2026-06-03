
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `at` method.
#test("Hello".at(1), "e")
#test("Hello".at(4), "o")
#test("Hello".at(-1), "o")
#test("Hello".at(-2), "l")
#test("Hey: 🏳️‍🌈 there!".at(5), "🏳️‍🌈")