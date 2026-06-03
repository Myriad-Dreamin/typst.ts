
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that integer -> integer conversion doesn't do a roundtrip through float.
#let x = 9223372036854775800
#test(type(x), int)
#test(int(x), x)