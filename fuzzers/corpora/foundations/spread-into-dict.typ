
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#{
  let x = (a: 1)
  let y = (b: 2)
  let z = (a: 3)
  test((:..x, ..y, ..z), (a: 3, b: 2))
  test((..(a: 1), b: 2), (a: 1, b: 2))
}