
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test negative indices.
#{
  let array = (1, 2, 3, 4)
  test(array.at(0), 1)
  test(array.at(-1), 4)
  test(array.at(-2), 3)
  test(array.at(-3), 2)
  test(array.at(-4), 1)
}