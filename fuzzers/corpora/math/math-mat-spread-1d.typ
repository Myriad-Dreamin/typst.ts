
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
$ mat(..#range(1, 5) ; 1, ..#range(2, 5))
  mat(..#range(1, 3), ..#range(3, 5) ; ..#range(1, 4), 4) $