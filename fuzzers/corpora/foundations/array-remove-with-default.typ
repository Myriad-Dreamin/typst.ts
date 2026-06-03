
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test remove with default value.

#{
  let array = (1, 2, 3)
  test(array.remove(2, default: 5), 3)
}

#{
  let array = (1, 2, 3)
  test(array.remove(3, default: 5), 5)
}