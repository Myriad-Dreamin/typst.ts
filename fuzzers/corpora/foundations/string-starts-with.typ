
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `starts-with` and `ends-with` methods.
#test("Typst".starts-with("Ty"), true)
#test("Typst".starts-with(regex("[Tt]ys")), false)
#test("Typst".starts-with("st"), false)