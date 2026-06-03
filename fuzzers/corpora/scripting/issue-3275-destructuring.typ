
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Destructuring.
#for (a,b,c) in (("a", 1, bytes(())), ("b", 2, bytes(""))) {}
#for (a, ..) in (("a", 1, bytes(())), ("b", 2, bytes(""))) {}
#for (k, v)  in (a: 1, b: 2, c: 3) {}
#for (.., v) in (a: 1, b: 2, c: 3) {}