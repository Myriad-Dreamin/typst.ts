
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Destructuring with a sink in the middle.
#let (a: _, ..b, c: _) = (a: 1, b: 2, c: 3)
#test(b, (b: 2))