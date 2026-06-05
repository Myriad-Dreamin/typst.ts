
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that matrices, vectors, and cases are all laid out the same.
$ mat(z_(n_p); a^2)
  vec(z_(n_p), a^2)
  cases(reverse: #true, delim: \(, z_(n_p), a^2)
  cases(delim: \(, z_(n_p), a^2) $