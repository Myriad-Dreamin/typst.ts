
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Spread at beginning.
#{
  let f(..a, b) = (a, b)
  test(repr(f(1)), "(arguments(), 1)")
  test(repr(f(1, 2, 3)), "(arguments(1, 2), 3)")
  test(repr(f(1, 2, 3, 4, 5)), "(arguments(1, 2, 3, 4), 5)")
}