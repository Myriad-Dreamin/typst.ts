
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Assignment is right-associative.
#{
  let x = 1
  let y = 2
  x = y = "ok"
  test(x, none)
  test(y, "ok")
}