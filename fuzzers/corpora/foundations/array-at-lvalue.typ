
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test lvalue and rvalue access.
#{
  let array = (1, 2)
  array.at(1) += 5 + array.at(0)
  test(array, (1, 8))
}