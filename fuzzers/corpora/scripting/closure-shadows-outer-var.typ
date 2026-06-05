
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Redefined variable.
#{
  let x = 1
  let f() = {
    let x = x + 2
    x
  }
  test(f(), 3)
}