
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// For loop bindings.
#{
  let v = (1, 2, 3)
  let f() = {
    let s = 0
    for v in v { s += v }
    s
  }
  test(f(), 6)
}