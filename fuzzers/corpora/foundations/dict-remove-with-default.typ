
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test remove with default value.
#{
  let dict = (a: 1, b: 2)
  test(dict.remove("b", default: 3), 2)
}

#{
  let dict = (a: 1, b: 2)
  test(dict.remove("c", default: 3), 3)
}