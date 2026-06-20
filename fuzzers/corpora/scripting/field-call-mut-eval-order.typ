
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test evaluation order of mutating methods with accessors.
#{
  let pair = (1, 2)
  let arrays = ((), (), ())
  arrays.at(pair.remove(0)).push(pair.remove(0))
  //             ^^^^^^ second (2)    ^^^^^^ first (1)
  test(arrays, ((), (), (1,)))
}