
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `to-dict` method.
#test(().to-dict(), (:))
#test((("a", 1), ("b", 2), ("c", 3)).to-dict(), (a: 1, b: 2, c: 3))
#test((("a", 1), ("b", 2), ("c", 3), ("b", 4)).to-dict(), (a: 1, b: 4, c: 3))