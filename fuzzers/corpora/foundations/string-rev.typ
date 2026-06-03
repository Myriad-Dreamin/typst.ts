
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `rev` method.
#test("abc".rev(), "cba")
#test("ax̂e".rev(), "ex̂a")