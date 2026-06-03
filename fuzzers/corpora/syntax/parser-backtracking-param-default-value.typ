
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#{
  let s = "(x: 1) => x"
  let pat = "(x: {}) => 1 + x()"
  for _ in range(50) {
    s = pat.replace("{}", s)
  }
  test(eval(s)(), 51)
}