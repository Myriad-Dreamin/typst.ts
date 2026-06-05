
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Parameter bindings.
#{
  let x = 5
  let g() = {
    let f(x, y: x) = x + y
    f
  }

  test(g()(8), 13)
}