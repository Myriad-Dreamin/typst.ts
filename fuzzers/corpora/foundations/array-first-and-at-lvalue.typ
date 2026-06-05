
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test different lvalue method.
#{
  let array = (1, 2, 3)
  array.first() = 7
  array.at(1) *= 8
  test(array, (7, 16, 3))
}