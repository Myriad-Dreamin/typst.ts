
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `first` and `last` methods.
#test("Hello".first(), "H")
#test("Hello".last(), "o")
#test("🏳️‍🌈A🏳️‍⚧️".first(), "🏳️‍🌈")
#test("🏳️‍🌈A🏳️‍⚧️".last(), "🏳️‍⚧️")
#test("hey".first(default: "d"), "h")
#test("".first(default: "d"), "d")
#test("hey".last(default: "d"), "y")
#test("".last(default: "d"), "d")