
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show math.vec: it => {
  show regex("\(|\)"): set text(blue)
  it
}
$ vec(1, 0, 0), mat(1; 0; 0), (1), binom(n, k) $