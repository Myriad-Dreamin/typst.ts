
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that removal keeps order.
#let dict = (a: 1, b: 2, c: 3, d: 4)
#dict.remove("b")
#test(dict.keys(), ("a", "c", "d"))