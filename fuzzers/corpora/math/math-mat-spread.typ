
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test argument spreading in matrix.
$ mat(..#range(1, 5).chunks(2))
  mat(#(..range(2).map(_ => range(2)))) $

#let nums = ((1,) * 5).intersperse(0).chunks(3)
$ mat(..nums, delim: "[") $