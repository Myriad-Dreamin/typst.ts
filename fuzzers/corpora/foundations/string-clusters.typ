
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `clusters` and `codepoints` methods.
#test("abc".clusters(), ("a", "b", "c"))
#test("abc".clusters(), ("a", "b", "c"))
#test("🏳️‍🌈!".clusters(), ("🏳️‍🌈", "!"))