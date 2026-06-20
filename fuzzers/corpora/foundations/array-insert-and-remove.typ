
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `insert` and `remove` methods.
#{
  let array = (0, 1, 2, 4, 5)
  array.insert(3, 3)
  test(array, range(6))
  _ = array.remove(1)
  test(array, (0, 2, 3, 4, 5))
}