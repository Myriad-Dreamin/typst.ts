
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Spread in the middle.
#{
  let f(a, ..b, c) = (a, b, c)
  test(repr(f(1, 2)), "(1, arguments(), 2)")
  test(repr(f(1, 2, 3, 4, 5)), "(1, arguments(2, 3, 4), 5)")
}