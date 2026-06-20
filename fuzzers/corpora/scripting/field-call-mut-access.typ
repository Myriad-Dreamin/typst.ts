
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test calling a mutating method from accessor methods.
#{
  let matrix = (((1,), (2,)), ((3,), (4,)))
  matrix.at(1).at(0).push(5)
  test(matrix, (((1,), (2,)), ((3, 5), (4,))))
}