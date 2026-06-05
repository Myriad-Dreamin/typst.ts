
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test spreading into array and dictionary.
#{
  let l = (1, 2, 3)
  let r = (5, 6, 7)
  test((..l, 4, ..r), range(1, 8))
  test((..none), ())
}